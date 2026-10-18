package org.fleetranger.rc

import android.Manifest
import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.net.ConnectivityManager
import android.net.Network
import android.os.Handler
import android.os.HandlerThread
import android.os.IBinder
import android.util.Log
import java.io.File
import java.io.IOException
import java.security.MessageDigest

/**
 * Uploads DJI Pilot 2's flight logs, as the prototype did: it looks through
 * the log folders every minute, and uploads a log once it hasn't changed for
 * a few minutes, over an approved Wi-Fi network that isn't metered. Each log
 * goes with the pilot chosen before it was written. A log that changes after
 * its upload (it was still being written) is uploaded again, and replaces the
 * earlier version on the server.
 */
class UploadService : Service() {
    private lateinit var worker: Handler
    private lateinit var server: Server
    private lateinit var pilots: Pilots
    private lateinit var sync: SyncSettings
    private lateinit var logs: FlightLogs
    private lateinit var updates: Updates
    /** When the last sync was reported to the server; 0: not since the service started. */
    private var lastSyncReport = 0L

    private val scan = Runnable { scan() }

    /** A new connection looks again straight away, not at the next minute. */
    private val networkWatch = object : ConnectivityManager.NetworkCallback() {
        override fun onAvailable(network: Network) {
            worker.removeCallbacks(scan)
            worker.post(scan)
        }
    }

    override fun onCreate() {
        super.onCreate()
        server = Server(this)
        pilots = Pilots(this)
        sync = SyncSettings(this)
        logs = FlightLogs(this)
        updates = Updates(this)
        getSystemService(NotificationManager::class.java).createNotificationChannel(
            NotificationChannel(CHANNEL, "Uploads", NotificationManager.IMPORTANCE_LOW),
        )
        startForeground(NOTIFICATION, notification("Starting…", busy = false))
        val thread = HandlerThread("uploads").apply { start() }
        worker = Handler(thread.looper)
        worker.post(scan)
        getSystemService(ConnectivityManager::class.java).registerDefaultNetworkCallback(networkWatch)
    }

    override fun onDestroy() {
        getSystemService(ConnectivityManager::class.java).unregisterNetworkCallback(networkWatch)
        worker.looper.quitSafely()
        super.onDestroy()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        // Opening the app looks again straight away, e.g. after approving a network.
        worker.removeCallbacks(scan)
        worker.post(scan)
        return START_STICKY
    }

    override fun onBind(intent: Intent?): IBinder? = null

    private fun scan() {
        worker.removeCallbacks(scan)
        val uploadedBefore = SyncStatus.read(this).uploaded
        try {
            uploadWaiting()
        } catch (error: Exception) {
            Log.w(TAG, "upload scan failed", error)
        }
        try {
            reportSync(uploaded = SyncStatus.read(this).uploaded != uploadedBefore)
        } catch (error: Exception) {
            Log.w(TAG, "sync report failed", error)
        }
        // This service runs all the time, so it also looks for updates daily.
        try {
            if (server.paired) updates.checkIfDue(server)
        } catch (error: Exception) {
            Log.w(TAG, "update check failed", error)
        }
        worker.postDelayed(scan, RESCAN_MS)
    }

    private fun uploadWaiting() {
        if (!server.paired) return report(SyncStatus.State.NotPaired, waiting = 0)
        if (checkSelfPermission(Manifest.permission.READ_EXTERNAL_STORAGE) !=
            PackageManager.PERMISSION_GRANTED
        ) {
            return report(SyncStatus.State.NoAccess, waiting = 0)
        }
        val waiting = logs.all()
            .filter { it.state == FlightLogs.State.Waiting || it.state == FlightLogs.State.BeingWritten }
            .reversed()
            .toMutableList()
        pilots.forgetBefore(waiting.firstOrNull()?.file?.lastModified() ?: System.currentTimeMillis())
        if (waiting.isEmpty()) return report(SyncStatus.State.Done, waiting = 0)
        if (!sync.mayUploadNow()) {
            return report(SyncStatus.State.WaitingForNetwork, waiting = waiting.size)
        }
        for (entry in waiting.toList()) {
            if (entry.state == FlightLogs.State.BeingWritten) continue
            val log = entry.file
            // Taken before reading, so a write during the upload counts as a change.
            val key = FlightLogs.key(log)
            report(SyncStatus.State.Uploading, waiting = waiting.size, current = log.name)
            val result = try {
                server.upload(log, sha256(log), pilots.at(log.lastModified()))
            } catch (error: IOException) {
                Log.w(TAG, "upload of ${log.name} failed: $error")
                return report(SyncStatus.State.Unreachable, waiting = waiting.size)
            }
            when (result) {
                Server.Upload.Stored -> {
                    logs.markUploaded(key)
                    waiting.remove(entry)
                    val before = SyncStatus.read(this)
                    SyncStatus.write(
                        this,
                        before.copy(uploaded = before.uploaded + 1, lastUploadAt = System.currentTimeMillis()),
                    )
                }
                Server.Upload.Refused -> Log.w(TAG, "the server refused ${log.name}")
                Server.Upload.Unpaired -> return report(SyncStatus.State.NotPaired, waiting = 0)
            }
        }
        // Logs still being written, or refused: they're tried again next time.
        report(if (waiting.isEmpty()) SyncStatus.State.Done else SyncStatus.State.Uploading, waiting.size)
    }

    /**
     * A sync is a scan that could upload: paired, the logs readable, and on a
     * connection the settings allow; also when there was nothing to upload.
     * The server shows when the controller last synced, so it's reported, at
     * most every [SYNC_REPORT_MS] and straight after uploads.
     */
    private fun reportSync(uploaded: Boolean) {
        val state = SyncStatus.read(this).state
        if (state == SyncStatus.State.NotPaired || state == SyncStatus.State.NoAccess) return
        if (!sync.mayUploadNow()) return
        val now = System.currentTimeMillis()
        if (!uploaded && now - lastSyncReport < SYNC_REPORT_MS) return
        server.reportSync(BuildConfig.VERSION_NAME)
        lastSyncReport = now
    }

    /** Tells the screens and the notification what the service is doing. */
    private fun report(state: SyncStatus.State, waiting: Int, current: String? = null) {
        val status = SyncStatus.read(this)
            .copy(state = state, waiting = waiting, current = current, uploadOver = sync.uploadOver)
        SyncStatus.write(this, status)
        val busy = state == SyncStatus.State.Uploading && current != null
        getSystemService(NotificationManager::class.java)
            .notify(NOTIFICATION, notification(status.summary(), busy, settings = state == SyncStatus.State.WaitingForNetwork))
    }

    /** The system's upload icon is animated, so it's only shown while uploading. */
    private fun notification(text: String, busy: Boolean, settings: Boolean = false): Notification =
        Notification.Builder(this, CHANNEL)
            .setSmallIcon(
                if (busy) android.R.drawable.stat_sys_upload else android.R.drawable.stat_sys_upload_done,
            )
            .setContentTitle("FleetRanger")
            .setContentText(text)
            .setOngoing(true)
            .setContentIntent(openApp(settings))
            .build()

    /** Tapping the notification opens the app; its settings when something needs fixing there. */
    private fun openApp(settings: Boolean): PendingIntent {
        val intent = Intent(this, MainActivity::class.java)
            .putExtra(MainActivity.EXTRA_SETTINGS, settings)
            .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_SINGLE_TOP)
        return PendingIntent.getActivity(
            this,
            if (settings) 1 else 0,
            intent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
        )
    }

    companion object {
        private const val TAG = "FleetRanger"
        private const val CHANNEL = "uploads"
        private const val NOTIFICATION = 1
        private const val RESCAN_MS = 60_000L
        private const val SYNC_REPORT_MS = 15 * 60_000L

        fun start(context: Context) {
            context.startForegroundService(Intent(context, UploadService::class.java))
        }

        private fun sha256(file: File): String {
            val digest = MessageDigest.getInstance("SHA-256")
            file.inputStream().use { input ->
                val buffer = ByteArray(64 * 1024)
                while (true) {
                    val read = input.read(buffer)
                    if (read < 0) break
                    digest.update(buffer, 0, read)
                }
            }
            return digest.digest().joinToString("") { "%02x".format(it) }
        }
    }
}
