package org.fleetranger.rc

import android.Manifest
import android.content.Context
import android.content.pm.PackageManager
import android.net.ConnectivityManager
import android.net.NetworkCapabilities
import android.net.wifi.WifiManager
import java.time.LocalDate

/**
 * When logs sync: only over approved Wi-Fi networks, never over a metered one,
 * and only logs from a chosen date on (all logs by default). See "Clients" in
 * docs/architecture.md.
 */
class SyncSettings(private val context: Context) {
    private val prefs = context.getSharedPreferences("sync", Context.MODE_PRIVATE)

    /** The names (SSIDs) of the approved Wi-Fi networks. */
    var networks: Set<String>
        get() = prefs.getStringSet("networks", emptySet()).orEmpty().toSortedSet()
        set(value) = prefs.edit().putStringSet("networks", value).apply()

    /** Logs written before this day don't sync; null syncs all logs. */
    var syncFrom: LocalDate?
        get() = prefs.getString("sync_from", null)?.let(LocalDate::parse)
        set(value) = prefs.edit().putString("sync_from", value?.toString()).apply()

    /**
     * The Wi-Fi network the controller is on, or null if it isn't on Wi-Fi or
     * Android doesn't reveal the name: that needs the location permission.
     */
    fun currentNetwork(): String? {
        if (!canSeeNetworkNames()) return null
        val connectivity = context.getSystemService(ConnectivityManager::class.java)
        val capabilities = connectivity.getNetworkCapabilities(connectivity.activeNetwork)
        if (capabilities?.hasTransport(NetworkCapabilities.TRANSPORT_WIFI) != true) return null
        @Suppress("DEPRECATION") // The replacement needs Android 12; the controllers run 10.
        val ssid = context.getSystemService(WifiManager::class.java).connectionInfo?.ssid
        return ssid?.removeSurrounding("\"")?.takeIf { it.isNotEmpty() && it != UNKNOWN_SSID }
    }

    fun canSeeNetworkNames(): Boolean =
        context.checkSelfPermission(Manifest.permission.ACCESS_FINE_LOCATION) ==
            PackageManager.PERMISSION_GRANTED

    private companion object {
        /** What Android reports when it hides the network's name. */
        const val UNKNOWN_SSID = "<unknown ssid>"
    }
}
