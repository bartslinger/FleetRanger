package org.fleetranger.rc

import android.content.Context
import android.content.SharedPreferences

/**
 * What the upload service is doing, for the screens: it writes, they read
 * and follow changes. Kept in the app's storage, so it survives restarts.
 */
data class SyncStatus(
    val state: State,
    /** Logs waiting to upload. */
    val waiting: Int,
    /** Logs uploaded, ever. */
    val uploaded: Int,
    /** Epoch milliseconds of the last upload; 0 if none yet. */
    val lastUploadAt: Long,
    /** The log being uploaded. */
    val current: String?,
    /** The setting at the time, for what logs wait for. */
    val uploadOver: SyncSettings.UploadOver = SyncSettings.UploadOver.AnyWifi,
) {
    enum class State {
        /** No scan yet. */
        Starting,
        NotPaired,
        /** The flight logs can't be read: the storage permission is missing. */
        NoAccess,
        /** No connection that the settings allow uploading over. */
        WaitingForNetwork,
        Uploading,
        /** The server can't be reached; trying again soon. */
        Unreachable,
        Done,
    }

    /** Logs aren't uploading: shown with a warning icon, so it isn't missed. */
    val problem: Boolean
        get() = state in setOf(State.NotPaired, State.NoAccess, State.WaitingForNetwork, State.Unreachable)

    /** One line for the pilot screen and the settings. */
    fun summary(): String = when (state) {
        State.Starting -> "Starting"
        State.NotPaired -> "Not paired"
        State.NoAccess -> "Can’t read the flight logs: allow access to files"
        State.WaitingForNetwork -> "${plural(waiting, "log")} waiting for " + when (uploadOver) {
            SyncSettings.UploadOver.ApprovedWifi -> "an approved Wi-Fi network"
            SyncSettings.UploadOver.AnyWifi -> "a Wi-Fi network that isn’t a hotspot"
            SyncSettings.UploadOver.AnyConnection -> "a connection"
        }
        State.Uploading -> "Uploading ${current ?: "a log"} (${waiting} waiting)"
        State.Unreachable -> "Can’t reach FleetRanger; ${plural(waiting, "log")} waiting"
        // Nothing uploaded yet, e.g. every log is from before the chosen date.
        State.Done -> if (uploaded == 0) "No logs to upload" else "All logs uploaded"
    }

    companion object {
        private const val PREFS = "sync_status"

        fun read(context: Context): SyncStatus {
            val prefs = prefs(context)
            return SyncStatus(
                state = prefs.getString("state", null)
                    ?.let { runCatching { State.valueOf(it) }.getOrNull() } ?: State.Starting,
                waiting = prefs.getInt("waiting", 0),
                uploaded = prefs.getInt("uploaded", 0),
                lastUploadAt = prefs.getLong("last_upload_at", 0),
                current = prefs.getString("current", null),
                uploadOver = prefs.getString("upload_over", null)
                    ?.let { runCatching { SyncSettings.UploadOver.valueOf(it) }.getOrNull() }
                    ?: SyncSettings.UploadOver.AnyWifi,
            )
        }

        fun write(context: Context, status: SyncStatus) {
            prefs(context).edit()
                .putString("state", status.state.name)
                .putInt("waiting", status.waiting)
                .putInt("uploaded", status.uploaded)
                .putLong("last_upload_at", status.lastUploadAt)
                .putString("current", status.current)
                .putString("upload_over", status.uploadOver.name)
                .apply()
        }

        /** Calls `onChange` whenever the service writes; keep the returned listener. */
        fun listen(context: Context, onChange: (SyncStatus) -> Unit): SharedPreferences.OnSharedPreferenceChangeListener {
            val listener = SharedPreferences.OnSharedPreferenceChangeListener { _, _ ->
                onChange(read(context))
            }
            prefs(context).registerOnSharedPreferenceChangeListener(listener)
            return listener
        }

        fun stopListening(context: Context, listener: SharedPreferences.OnSharedPreferenceChangeListener) {
            prefs(context).unregisterOnSharedPreferenceChangeListener(listener)
        }

        private fun prefs(context: Context) =
            context.getSharedPreferences(PREFS, Context.MODE_PRIVATE)

        private fun plural(count: Int, noun: String) = "$count $noun${if (count == 1) "" else "s"}"
    }
}
