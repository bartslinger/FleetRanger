package org.fleetranger.rc

import android.content.Context
import android.content.Intent
import android.provider.Settings

/** DJI Pilot 2, the flight app FleetRanger hands over to. */
object DjiPilot {
    private const val PACKAGE = "com.dji.industry.pilot"

    /** Opens DJI Pilot 2; false if it isn't installed. */
    fun open(context: Context): Boolean {
        val intent = context.packageManager.getLaunchIntentForPackage(PACKAGE) ?: return false
        context.startActivity(intent)
        return true
    }

    /**
     * Whether the controller opens FleetRanger when it starts, as set under
     * Settings > Apps & notifications > Auto start application selection. DJI
     * keeps the choice in a system property.
     */
    fun opensAtStartup(context: Context): Boolean = try {
        val process = ProcessBuilder("getprop", "persist.dji.default_app").start()
        process.inputStream.bufferedReader().use { it.readLine()?.trim() } == context.packageName
    } catch (_: Exception) {
        false
    }

    /** The controller's settings, where the startup app is chosen. */
    fun openSettings(context: Context) {
        context.startActivity(Intent(Settings.ACTION_SETTINGS))
    }
}
