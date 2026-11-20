package org.fleetranger.rc

import android.app.Application

/** Starts error reporting before anything else runs. */
class FleetRangerApp : Application() {
    override fun onCreate() {
        super.onCreate()
        ErrorReports.start(this)
    }
}
