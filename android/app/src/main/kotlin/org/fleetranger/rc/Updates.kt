package org.fleetranger.rc

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.SharedPreferences
import android.content.pm.PackageInstaller
import java.io.IOException

/**
 * Updates of this app. The server offers the latest release's APK (see
 * "Releases" in android/README.md); the upload service checks once a day, the
 * settings on request. Android's installer installs it: the person holding the
 * controller confirms, and the first time allows FleetRanger to install apps.
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

    /** Whether the last update didn't install, e.g. it was cancelled. */
    val failed: Boolean
        get() = prefs.getBoolean("failed", false)

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
     * Downloads the release straight into an installer session and commits
     * it; Android then asks to confirm ([InstallResult]). Blocks.
     */
    fun install(server: Server) {
        prefs.edit().putBoolean("failed", false).apply()
        val installer = context.packageManager.packageInstaller
        val session = installer.createSession(
            PackageInstaller.SessionParams(PackageInstaller.SessionParams.MODE_FULL_INSTALL),
        )
        try {
            installer.openSession(session).use { writing ->
                server.downloadApp { input, length ->
                    writing.openWrite("FleetRanger.apk", 0, length).use { output ->
                        input.copyTo(output)
                        writing.fsync(output)
                    }
                }
                val result = PendingIntent.getBroadcast(
                    context,
                    session,
                    Intent(context, InstallResult::class.java),
                    // Mutable: the installer adds its status to it.
                    PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_MUTABLE,
                )
                writing.commit(result.intentSender)
            }
        } catch (error: IOException) {
            installer.abandonSession(session)
            throw error
        }
    }

    companion object {
        private const val PREFS = "updates"
        private const val DAY_MS = 24 * 3_600_000L
        private const val CHANNEL = "updates"
        private const val NOTIFICATION = 2

        /**
         * After an update Android doesn't open the app again, and Android 10
         * doesn't let it bring itself to the front: a notification says the
         * update worked and opens it.
         */
        fun notifyInstalled(context: Context) {
            val notifications = context.getSystemService(NotificationManager::class.java)
            notifications.createNotificationChannel(
                NotificationChannel(CHANNEL, "App updates", NotificationManager.IMPORTANCE_DEFAULT),
            )
            val open = PendingIntent.getActivity(
                context,
                0,
                Intent(context, MainActivity::class.java).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK),
                PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
            )
            notifications.notify(
                NOTIFICATION,
                Notification.Builder(context, CHANNEL)
                    .setSmallIcon(android.R.drawable.stat_sys_download_done)
                    .setContentTitle("FleetRanger updated to ${BuildConfig.VERSION_NAME}")
                    .setContentText("Tap to open FleetRanger")
                    .setContentIntent(open)
                    .setAutoCancel(true)
                    .build(),
            )
        }

        /** Calls `onChange` whenever a check or install changes something; keep the listener. */
        fun listen(context: Context, onChange: () -> Unit): SharedPreferences.OnSharedPreferenceChangeListener {
            val listener = SharedPreferences.OnSharedPreferenceChangeListener { _, _ -> onChange() }
            context.getSharedPreferences(PREFS, Context.MODE_PRIVATE).registerOnSharedPreferenceChangeListener(listener)
            return listener
        }

        fun stopListening(context: Context, listener: SharedPreferences.OnSharedPreferenceChangeListener) {
            context.getSharedPreferences(PREFS, Context.MODE_PRIVATE).unregisterOnSharedPreferenceChangeListener(listener)
        }
    }

    /**
     * What Android's installer says about an update: it needs the person to
     * confirm (shown straight away), or it didn't install. When it did, Android
     * replaces the app, and [BootReceiver] starts the uploads again.
     */
    class InstallResult : BroadcastReceiver() {
        override fun onReceive(context: Context, intent: Intent) {
            when (intent.getIntExtra(PackageInstaller.EXTRA_STATUS, PackageInstaller.STATUS_FAILURE)) {
                PackageInstaller.STATUS_PENDING_USER_ACTION -> {
                    @Suppress("DEPRECATION") // The typed version needs Android 13; the controllers run 10.
                    val confirm = intent.getParcelableExtra<Intent>(Intent.EXTRA_INTENT) ?: return
                    context.startActivity(confirm.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK))
                }
                PackageInstaller.STATUS_SUCCESS -> Unit
                else -> context.getSharedPreferences(PREFS, Context.MODE_PRIVATE).edit()
                    .putBoolean("failed", true).apply()
            }
        }
    }
}
