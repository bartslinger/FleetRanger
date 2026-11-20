package org.fleetranger.rc

import android.content.Context
import io.sentry.Sentry
import io.sentry.SentryLevel
import io.sentry.android.core.SentryAndroid
import java.io.IOException

/**
 * Crashes and unexpected errors go to the error tracker the server names
 * (`GET /settings`), never anywhere else: no address is built into the app,
 * and without one from the server nothing is sent. Reports carry the error,
 * the device and this controller's id on the server, nothing personal. See
 * "Clients" in docs/architecture.md.
 */
object ErrorReports {
    private const val DAY_MS = 24 * 60 * 60 * 1000L

    /** When this process last fetched the settings; 0: not yet, so at start. */
    private var fetchedAt = 0L

    /**
     * Starts reporting with the settings fetched before, at the app's start:
     * a crash early on is reported too, and changed settings count from the
     * next start.
     */
    fun start(context: Context) {
        val prefs = prefs(context)
        val dsn = prefs.getString("dsn", null) ?: return
        val controller = prefs.getString("controller", null) ?: return
        if (!Server(context).paired) return
        SentryAndroid.init(context) { options ->
            options.dsn = dsn
            // Personal data (user, IP address) stays off: the SDK's default.
            options.release = "org.fleetranger.rc@${BuildConfig.VERSION_NAME}+${BuildConfig.VERSION_CODE}"
            options.environment = if (BuildConfig.DEBUG) "debug" else "release"
            options.setTag("controller", controller)
        }
    }

    /** Fetches the settings at start and daily, from the upload service. */
    fun refreshIfDue(context: Context, server: Server) {
        val now = System.currentTimeMillis()
        if (fetchedAt != 0L && now - fetchedAt < DAY_MS) return
        val settings = try {
            server.settings()
        } catch (_: IOException) {
            return // Not reachable now: try again next scan.
        }
        fetchedAt = now
        val edit = prefs(context).edit()
        // Null also from an older server without settings, or an unpaired controller.
        if (settings?.errorReports == null) {
            edit.remove("dsn").remove("controller")
        } else {
            edit.putString("dsn", settings.errorReports).putString("controller", settings.controller)
        }
        edit.putString("support", settings?.support).apply()
    }

    /** Whom pilots contact about the flight logs, as the server said; null: nobody named. */
    fun support(context: Context): String? = prefs(context).getString("support", null)

    /** An error the app caught but didn't expect; network trouble isn't one. */
    fun report(error: Throwable) {
        if (error is IOException) return
        if (Sentry.isEnabled()) Sentry.captureException(error)
    }

    /**
     * Reports the log folders' state when it begins, not on every scan: again
     * only after it went away or changed. Unreadable is a warning; none found
     * is information, as a controller that never flew has none either.
     */
    fun folders(context: Context, check: FlightLogs.Check) {
        // Without reporting, it counts as not yet reported.
        if (!Sentry.isEnabled()) return
        val prefs = prefs(context)
        val (message, level) = when (check.access) {
            FlightLogs.Access.Found -> null to null
            FlightLogs.Access.NotFound -> "No FlightRecord folder found" to SentryLevel.INFO
            FlightLogs.Access.Unreadable -> "A flight log folder can't be read" to SentryLevel.WARNING
        }
        val now = message?.let { "$it ${check.places}" }
        if (now == prefs.getString("folder_problem", null)) return
        if (message != null && level != null) send(context, message, level, check.places)
        prefs.edit().putString("folder_problem", now).apply()
    }

    /**
     * A pilot searched the controller for the logs: where they were (so the
     * app can look there by default), or that there were none.
     */
    fun searched(context: Context, found: String?, check: FlightLogs.Check) {
        if (!Sentry.isEnabled()) return
        val places = check.places + ("found" to (found ?: "nothing"))
        val message = if (found != null) "Flight logs found elsewhere" else "A search found no flight logs"
        send(context, message, SentryLevel.INFO, places)
    }

    private fun send(context: Context, message: String, level: SentryLevel, places: Map<String, String>) {
        Sentry.captureMessage(message, level) { scope ->
            places.forEach { (place, state) -> scope.setExtra(place, state) }
            scope.setExtra("DJI Pilot 2", DjiPilot.version(context) ?: "not installed")
        }
    }

    private fun prefs(context: Context) =
        context.getSharedPreferences("error_reports", Context.MODE_PRIVATE)
}
