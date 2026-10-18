package org.fleetranger.rc

import android.content.Context
import android.content.Intent
import android.content.SharedPreferences
import androidx.core.content.FileProvider
import java.io.File
import java.io.IOException

/**
 * Updates of this app. The server offers the latest release's APK (see
 * "Releases" in android/README.md); the upload service checks once a day, the
 * settings on request. Android's installer installs it, the same screens as
 * installing from the download page: the person holding the controller
 * confirms (the first time also allows FleetRanger to install apps), and
 * afterwards "App installed" opens the new version.
 */
class Updates(context: Context) {
    private val context = context.applicationContext
    private val prefs = context.getSharedPreferences(PREFS, Context.MODE_PRIVATE)

    /** A newer release than this app, as last checked; null if none. */
    val available: Server.AppVersion?
        get() {
            val code = prefs.getInt("code", 0)
            if (code <= BuildConfig.VERSION_CODE) return null
            return Server.AppVersion(code, prefs.getString("name", null).orEmpty())
        }

    /** Asks the server; blocks. An IOException if it can't be reached. */
    fun check(server: Server): Server.AppVersion? {
        val offered = server.offeredApp()
        prefs.edit()
            .putLong("checked_at", System.currentTimeMillis())
            .putInt("code", offered?.code ?: 0)
            .putString("name", offered?.name)
            .apply()
        return available
    }

    /** Once a day, from the upload service; offline it waits for the next time. */
    fun checkIfDue(server: Server) {
        if (System.currentTimeMillis() - prefs.getLong("checked_at", 0) < DAY_MS) return
        try {
            check(server)
        } catch (_: IOException) {
            // Not reachable now.
        }
    }

    /**
     * Downloads the release into the app's cache and returns the intent that
     * opens it in Android's installer; start it from the screen. Blocks.
     */
    fun download(server: Server): Intent {
        val folder = File(context.cacheDir, "updates").apply { mkdirs() }
        val apk = File(folder, "FleetRanger.apk")
        val part = File(folder, "FleetRanger.apk.part")
        server.downloadApp { input, _ -> part.outputStream().use { input.copyTo(it) } }
        if (!part.renameTo(apk)) throw IOException("the download couldn't be kept")
        val uri = FileProvider.getUriForFile(context, "${context.packageName}.updates", apk)
        return Intent(Intent.ACTION_VIEW)
            .setDataAndType(uri, "application/vnd.android.package-archive")
            .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
    }

    companion object {
        private const val PREFS = "updates"
        private const val DAY_MS = 24 * 3_600_000L

        /** Calls `onChange` whenever a check changes something; keep the listener. */
        fun listen(context: Context, onChange: () -> Unit): SharedPreferences.OnSharedPreferenceChangeListener {
            val listener = SharedPreferences.OnSharedPreferenceChangeListener { _, _ -> onChange() }
            context.getSharedPreferences(PREFS, Context.MODE_PRIVATE).registerOnSharedPreferenceChangeListener(listener)
            return listener
        }

        fun stopListening(context: Context, listener: SharedPreferences.OnSharedPreferenceChangeListener) {
            context.getSharedPreferences(PREFS, Context.MODE_PRIVATE).unregisterOnSharedPreferenceChangeListener(listener)
        }
    }
}
