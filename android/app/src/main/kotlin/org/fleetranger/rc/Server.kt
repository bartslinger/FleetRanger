package org.fleetranger.rc

import android.content.Context
import android.os.Build
import android.util.Base64
import java.io.File
import java.io.IOException
import java.io.InputStream
import java.net.HttpURLConnection
import java.net.URL
import java.security.MessageDigest
import java.security.SecureRandom
import org.json.JSONArray
import org.json.JSONException
import org.json.JSONObject

/**
 * The FleetRanger install this controller belongs to: its address, and the
 * token the controller uses once a user has approved pairing. See "Clients" in
 * docs/architecture.md. Calls block, so run them off the main thread.
 */
class Server(private val context: Context) {
    private val prefs = context.getSharedPreferences("server", Context.MODE_PRIVATE)

    /**
     * E.g. https://fleet.example.org; null until the download page's link set
     * it up or someone typed it on the pairing screen.
     * Another install means pairing again, so changing it forgets the token.
     */
    var url: String?
        get() = prefs.getString("url", null)
        set(value) {
            if (value == url) return
            prefs.edit().putString("url", value).remove("token").putBoolean("paired", false).apply()
        }

    /** Whether a user approved this controller, as last checked. */
    var paired: Boolean
        get() = prefs.getBoolean("paired", false)
        private set(value) = prefs.edit().putBoolean("paired", value).apply()

    private val token: String?
        get() = prefs.getString("token", null)

    /** Where this controller's token stands, as the server sees it. */
    enum class Check { Paired, Waiting, Unknown }

    /**
     * Asks to pair, with a new token: only its hash is sent, so the token never
     * leaves the controller. Returns the code a user approves in the web app.
     */
    fun startPairing(): String {
        val token = newToken()
        prefs.edit().putString("token", token).putBoolean("paired", false).apply()
        // Perhaps another install, or this one after its logs moved.
        FlightLogs(context).forgetUploaded()
        val body = JSONObject()
            .put("serial", serial())
            .put("model", Build.MODEL.replace('_', ' '))
            .put("token_sha256", sha256Hex(token))
        val answer = call("POST", "/api/pairing", body)
            ?: throw IOException("the server refused the pairing request")
        return answer.optString("code").ifEmpty { throw IOException("the server gave no pairing code") }
    }

    /**
     * Asks the server about this controller's token. While waiting for
     * approval, asking keeps the code valid; the server drops it about a
     * minute after the controller stops asking.
     */
    fun check(): Check {
        val answer = call("GET", "/api/controller")
        val result = when {
            answer == null -> Check.Unknown
            answer.optBoolean("paired") -> Check.Paired
            else -> Check.Waiting
        }
        paired = result == Check.Paired
        return result
    }

    /**
     * Downloads the pilots this controller offers. Null if the server refused
     * the token: the controller was unpaired, and has to pair again.
     */
    fun pilots(): PilotList? {
        val answer = call("GET", "/api/controller/pilots")
        if (answer == null) paired = false
        return answer?.let(PilotList::fromJson)
    }

    /**
     * Which of these logs, by SHA-256 in hex, the server doesn't have yet; at
     * most [MAX_HASHES] at a time. Null if the server can't tell (unpaired, or
     * a version without the question): then upload them all, as before.
     */
    fun missing(sha256s: List<String>): Set<String>? {
        val answer = call("POST", "/api/controller/logs/missing", JSONObject().put("sha256", JSONArray(sha256s)))
            ?: return null
        val missing = answer.optJSONArray("missing") ?: throw IOException("the server's answer isn't FleetRanger's")
        return (0 until missing.length()).map { missing.getString(it) }.toSet()
    }

    /** What became of an upload. */
    enum class Upload { Stored, Refused, Unpaired }

    /**
     * Uploads a flight log with its hash and the chosen pilot (null: none).
     * Refused means the server didn't take it (e.g. it changed while being
     * sent); try again later. No connection is an IOException.
     */
    fun upload(file: File, sha256: String, pilotId: String?): Upload {
        val base = url ?: throw IOException("no server set up")
        val connection = URL("$base/api/controller/logs").openConnection() as HttpURLConnection
        try {
            connection.requestMethod = "POST"
            connection.connectTimeout = 10_000
            connection.readTimeout = 60_000
            connection.doOutput = true
            connection.setFixedLengthStreamingMode(file.length())
            connection.setRequestProperty("Content-Type", "application/octet-stream")
            token?.let { connection.setRequestProperty("Authorization", "Bearer $it") }
            connection.setRequestProperty("X-File-Name", file.name)
            connection.setRequestProperty("X-SHA256", sha256)
            pilotId?.let { connection.setRequestProperty("X-Pilot-Id", it) }
            file.inputStream().use { input ->
                connection.outputStream.use { output -> input.copyTo(output) }
            }
            return when (connection.responseCode) {
                in 200..299 -> Upload.Stored
                401 -> {
                    paired = false
                    Upload.Unpaired
                }
                else -> Upload.Refused
            }
        } finally {
            connection.disconnect()
        }
    }

    /**
     * Tells the server this controller synced: it looked for logs on a
     * connection its settings allow (and uploaded any), with this app version.
     */
    fun reportSync(appVersion: String) {
        call("POST", "/api/controller/sync", JSONObject().put("app_version", appVersion))
            ?: throw IOException("the server refused the sync report")
    }

    /** A release of this app, as the server offers it. */
    data class AppVersion(val code: Int, val name: String)

    /**
     * The release the server's download page offers; null if it has none, or
     * doesn't know its version.
     */
    fun offeredApp(): AppVersion? {
        val android = call("GET", "/api/download")?.optJSONObject("android") ?: return null
        if (android.isNull("version_code")) return null
        return AppVersion(android.getInt("version_code"), android.optString("version_name"))
    }

    /** Downloads the offered APK and hands it to `read` with its length. */
    fun <T> downloadApp(read: (InputStream, Long) -> T): T {
        val base = url ?: throw IOException("no server set up")
        val connection = URL("$base/api/download/FleetRanger.apk").openConnection() as HttpURLConnection
        try {
            connection.connectTimeout = 10_000
            connection.readTimeout = 60_000
            if (connection.responseCode != 200) throw IOException("the server answered ${connection.responseCode}")
            return connection.inputStream.use { read(it, connection.contentLengthLong) }
        } finally {
            connection.disconnect()
        }
    }

    /** Unpairs this controller: its token stops working. */
    fun unpair() {
        try {
            call("DELETE", "/api/controller")
        } finally {
            prefs.edit().remove("token").putBoolean("paired", false).apply()
        }
    }

    /**
     * Calls the API. Returns the JSON answer (empty for no content), or null if
     * the server refused the token or the request (401, 404). Anything else,
     * such as no connection, is an IOException.
     */
    private fun call(method: String, path: String, body: JSONObject? = null): JSONObject? {
        val base = url ?: throw IOException("no server set up")
        val connection = URL(base + path).openConnection() as HttpURLConnection
        try {
            connection.requestMethod = method
            connection.connectTimeout = 10_000
            connection.readTimeout = 10_000
            token?.let { connection.setRequestProperty("Authorization", "Bearer $it") }
            if (body != null) {
                connection.doOutput = true
                connection.setRequestProperty("Content-Type", "application/json")
                connection.outputStream.use { it.write(body.toString().toByteArray()) }
            }
            return when (val status = connection.responseCode) {
                in 200..299 -> {
                    val text = connection.inputStream.bufferedReader().use { it.readText() }
                    // Something else than FleetRanger answers with something else than JSON.
                    try {
                        if (text.isBlank()) JSONObject() else JSONObject(text)
                    } catch (_: JSONException) {
                        throw IOException("the server's answer isn't FleetRanger's")
                    }
                }
                401, 404 -> null
                else -> throw IOException("the server answered $status")
            }
        } finally {
            connection.disconnect()
        }
    }

    companion object {
        /** The most logs [missing] may ask about at once; the server's limit. */
        const val MAX_HASHES = 1000

        private fun newToken(): String {
            val bytes = ByteArray(32).also { SecureRandom().nextBytes(it) }
            return Base64.encodeToString(bytes, Base64.URL_SAFE or Base64.NO_PADDING or Base64.NO_WRAP)
        }

        private fun sha256Hex(text: String): String =
            MessageDigest.getInstance("SHA-256").digest(text.toByteArray())
                .joinToString("") { "%02x".format(it) }

        private fun serial(): String =
            Serial.read().ifEmpty { throw IOException("the controller's serial number can't be read") }
    }
}

/** The controller's serial number, as in its flight logs. */
object Serial {
    /** Empty if it can't be read. */
    fun read(): String = try {
        ProcessBuilder("getprop", "ro.serialno").start().inputStream.bufferedReader()
            .use { it.readLine()?.trim().orEmpty() }
    } catch (_: IOException) {
        ""
    }
}
