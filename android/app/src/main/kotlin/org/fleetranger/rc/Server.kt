package org.fleetranger.rc

import android.content.Context
import android.os.Build
import android.util.Base64
import java.io.File
import java.io.IOException
import java.io.InputStream
import java.net.HttpURLConnection
import java.net.URL
import java.net.URLEncoder
import java.security.MessageDigest
import java.time.Instant
import java.security.SecureRandom
import org.json.JSONArray
import org.json.JSONException
import org.json.JSONObject

/**
 * The FleetRanger install this controller belongs to: its address, and the
 * token the controller uses once it paired with a code. See "Clients" in
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

    /** Whether this controller is paired, as last checked. */
    var paired: Boolean
        get() = prefs.getBoolean("paired", false)
        private set(value) = prefs.edit().putBoolean("paired", value).apply()

    private val token: String?
        get() = prefs.getString("token", null)

    /** The server refused a request, saying why in words for the person. */
    class Refused(message: String) : IOException(message)

    /**
     * When this controller last synced since it paired (looked for logs on a
     * connection its settings allow); null before the first sync.
     */
    var lastSync: Long?
        get() = prefs.getLong("last_sync", 0).takeIf { it > 0 }
        set(value) = prefs.edit().putLong("last_sync", value ?: 0).apply()

    /**
     * Pairs with a code a user created in the web app, and a new token: only
     * its hash is sent, so the token never leaves the controller. Returns the
     * group the controller joined, null for the organisation. Throws [Refused]
     * for a code that doesn't pair, and another IOException when no
     * FleetRanger answered.
     */
    fun pair(code: String): String? {
        val token = newToken()
        prefs.edit().putString("token", token).putBoolean("paired", false).remove("last_sync").apply()
        val body = JSONObject()
            .put("code", code)
            .put("serial", serial())
            .put("token_sha256", sha256Hex(token))
        val answer = call("POST", "$API/pairing", body)
            ?: throw IOException("no FleetRanger answered")
        // Perhaps another install, or this one after its logs moved.
        FlightLogs(context).forgetUploaded()
        paired = true
        return answer.optJSONObject("group")?.getString("name")
    }

    /**
     * Downloads the pilots this controller offers. Null if the server refused
     * the token: the controller was unpaired, and has to pair again.
     */
    fun pilots(): PilotList? {
        val answer = call("GET", "$API/pilots")
        if (answer == null) paired = false
        return answer?.let(PilotList::fromJson)
    }

    /**
     * Which of these logs, by SHA-256 in hex, the server doesn't have yet; at
     * most [MAX_HASHES] at a time. Null if the server can't tell (unpaired):
     * then upload them all, and the server sorts out duplicates.
     */
    fun missing(sha256s: List<String>): Set<String>? {
        val answer = call("POST", "$API/logs/check", JSONObject().put("sha256", JSONArray(sha256s)))
            ?: return null
        val missing = answer.optJSONArray("missing") ?: throw IOException("the server's answer isn't FleetRanger's")
        return (0 until missing.length()).map { missing.getString(it) }.toSet()
    }

    /** What became of an upload. */
    enum class Upload { Stored, Refused, Unpaired }

    /**
     * Uploads a flight log, put at its name, with its hash and who was chosen
     * (null: nobody): the pilot, and the group the flights belong to. Sending
     * it again is harmless. Refused means the server didn't take it (e.g. it
     * changed while being sent); try again later. No connection is an
     * IOException.
     */
    fun upload(file: File, sha256: String, chosen: Chosen?): Upload {
        val base = url ?: throw IOException("no server set up")
        // DJI's names have square brackets; URLEncoder writes a space as +.
        val name = URLEncoder.encode(file.name, "UTF-8").replace("+", "%20")
        val connection = URL("$base$API/logs/$name").openConnection() as HttpURLConnection
        try {
            connection.requestMethod = "PUT"
            connection.connectTimeout = 10_000
            connection.readTimeout = 60_000
            connection.doOutput = true
            connection.setFixedLengthStreamingMode(file.length())
            connection.setRequestProperty("User-Agent", USER_AGENT)
            connection.setRequestProperty("Content-Type", "application/octet-stream")
            token?.let { connection.setRequestProperty("Authorization", "Bearer $it") }
            connection.setRequestProperty("X-SHA256", sha256)
            chosen?.let { connection.setRequestProperty("X-Pilot-Id", it.pilotId) }
            chosen?.groupId?.let { connection.setRequestProperty("X-Group-Id", it) }
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
     * Tells the server this controller's status: this app version, and when
     * it last synced. The whole status every time, as it replaces the last.
     */
    fun reportStatus(appVersion: String) {
        val status = JSONObject().put("app_version", appVersion)
        lastSync?.let { status.put("last_sync", Instant.ofEpochMilli(it).toString()) }
        call("PUT", "$API/status", status) ?: throw IOException("the server refused the status")
    }

    /** A release of this app, as the server offers it. */
    data class AppVersion(val code: Int, val name: String)

    /** The release the server offers; null if it has none. */
    fun offeredApp(): AppVersion? {
        val release = call("GET", "$API/release") ?: return null
        return AppVersion(release.getInt("version_code"), release.getString("version_name"))
    }

    /** Downloads the offered APK and hands it to `read` with its length. */
    fun <T> downloadApp(read: (InputStream, Long) -> T): T {
        val base = url ?: throw IOException("no server set up")
        val connection = URL("$base$API/release/FleetRanger.apk").openConnection() as HttpURLConnection
        try {
            connection.setRequestProperty("User-Agent", USER_AGENT)
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
            call("DELETE", "$API/pairing")
        } finally {
            prefs.edit().remove("token").putBoolean("paired", false).apply()
        }
    }

    /**
     * Calls the API. Returns the JSON answer (empty for no content), or null if
     * the server refused the token or the request (401, 404). A request the
     * server finds wrong (400) is [Refused], with the server's message, as is
     * one turned away for too many attempts (429). Anything else, such as no
     * connection, is an IOException.
     */
    private fun call(method: String, path: String, body: JSONObject? = null): JSONObject? {
        val base = url ?: throw IOException("no server set up")
        val connection = URL(base + path).openConnection() as HttpURLConnection
        try {
            connection.requestMethod = method
            connection.setRequestProperty("User-Agent", USER_AGENT)
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
                400 -> throw Refused(message(connection) ?: "The server refused the request.")
                // nginx limits pairing attempts per address.
                429 -> throw Refused("Too many attempts from this network. Wait a minute and try again.")
                else -> throw IOException("the server answered $status")
            }
        } finally {
            connection.disconnect()
        }
    }

    companion object {
        /** The most logs [missing] may ask about at once; the server's limit. */
        const val MAX_HASHES = 1000

        /** The controller's API, version 1; see "Clients" in docs/architecture.md. */
        private const val API = "/api/controller/v1"

        /** On every request, so the server's logs tell which app sent it. */
        private val USER_AGENT = "FleetRanger/${BuildConfig.VERSION_NAME} (${BuildConfig.VERSION_CODE}) " +
            "Android ${Build.VERSION.RELEASE}; ${Build.MODEL}"

        /** The `message` of an error the server explains; null if it doesn't. */
        private fun message(connection: HttpURLConnection): String? = try {
            val text = connection.errorStream?.bufferedReader()?.use { it.readText() } ?: return null
            JSONObject(text).optString("message").ifBlank { null }
        } catch (_: JSONException) {
            null
        }

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
