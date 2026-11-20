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
        edit.apply()
    }

    /** An error the app caught but didn't expect; network trouble isn't one. */
    fun report(error: Throwable) {
        if (error is IOException) return
        if (Sentry.isEnabled()) Sentry.captureException(error)
    }

    /**
     * Reports a problem with the log folders when it begins, not on every
     * scan: again only after it went away, or became another one.
     */
    fun folderProblem(context: Context, problem: FlightLogs.FolderProblem?) {
        // Without reporting, it counts as not yet reported.
        if (!Sentry.isEnabled()) return
        val prefs = prefs(context)
        val reported = prefs.getString("folder_problem", null)
        val now = problem?.let { "${it.message} ${it.places}" }
        if (now == reported) return
        if (problem != null) {
            Sentry.captureMessage(problem.message, SentryLevel.WARNING) { scope ->
                problem.places.forEach { (place, state) -> scope.setExtra(place, state) }
                scope.setExtra("DJI Pilot 2", DjiPilot.version(context) ?: "not installed")
            }
        }
        prefs.edit().putString("folder_problem", now).apply()
    }

    private fun prefs(context: Context) =
        context.getSharedPreferences("error_reports", Context.MODE_PRIVATE)
}
