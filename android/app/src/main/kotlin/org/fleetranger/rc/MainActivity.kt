package org.fleetranger.rc

import android.Manifest
import android.content.Intent
import android.content.SharedPreferences
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Bundle
import android.provider.Settings
import android.view.WindowManager
import android.widget.Toast
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.lifecycleScope
import java.io.IOException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

/**
 * The app's screen: pairing until a user approved this controller, then who's
 * flying, then DJI Pilot 2. The controller opens it at startup once it is
 * chosen as the startup app.
 */
class MainActivity : ComponentActivity() {
    private lateinit var pilots: Pilots
    private lateinit var server: Server
    private var serverUrl by mutableStateOf<String?>(null)
    private var paired by mutableStateOf(false)
    private var opensAtStartup by mutableStateOf(true)
    private var pilotList by mutableStateOf(PilotList.EMPTY)
    private var last by mutableStateOf<Pilot?>(null)
    private lateinit var sync: SyncSettings
    private var showSettings by mutableStateOf(false)
    private var currentNetwork by mutableStateOf<String?>(null)
    private var canSeeNetworkNames by mutableStateOf(false)
    private var syncStatus by mutableStateOf<SyncStatus?>(null)
    /** Kept, or Android forgets the listener. */
    private var syncListener: SharedPreferences.OnSharedPreferenceChangeListener? = null

    /** Android only names the Wi-Fi network for apps with the location permission. */
    private val askForNetworkNames =
        registerForActivityResult(ActivityResultContracts.RequestPermission()) { refreshNetwork() }

    /** Reading DJI Pilot 2's flight logs needs the storage permission. */
    private var hasLogAccess by mutableStateOf(false)
    /** Asked before and not allowed: Android won't ask again. */
    private var logAccessRefused by mutableStateOf(false)
    private val askForLogs =
        registerForActivityResult(ActivityResultContracts.RequestPermission()) {
            getSharedPreferences("onboarding", MODE_PRIVATE).edit()
                .putBoolean("asked_for_logs", true).apply()
            startUploads()
        }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        // Like DJI Pilot 2, the screen stays on while the app is in front.
        window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
        pilots = Pilots(this)
        server = Server(this)
        sync = SyncSettings(this)
        syncStatus = SyncStatus.read(this)
        syncListener = SyncStatus.listen(this) { syncStatus = it }
        setUp(intent)
        serverUrl = server.url
        paired = server.paired
        setContent {
            when {
                !paired -> Pairing(server, serverUrl, onPaired = {
                    paired = true
                    refreshPilots()
                    startUploads()
                })
                !hasLogAccess -> LogAccessScreen(
                    refused = logAccessRefused,
                    onContinue = { askForLogs.launch(Manifest.permission.READ_EXTERNAL_STORAGE) },
                    onOpenAppSettings = {
                        startActivity(
                            Intent(
                                Settings.ACTION_APPLICATION_DETAILS_SETTINGS,
                                Uri.fromParts("package", packageName, null),
                            ),
                        )
                    },
                )
                showSettings -> SettingsScreen(
                    settings = sync,
                    serverUrl = serverUrl,
                    currentNetwork = currentNetwork,
                    canSeeNetworkNames = canSeeNetworkNames,
                    onAllowNetworkNames = {
                        askForNetworkNames.launch(Manifest.permission.ACCESS_FINE_LOCATION)
                    },
                    onUnpair = ::unpair,
                    onBack = { showSettings = false },
                    syncStatus = syncStatus,
                )
                else -> PilotScreen(
                    list = pilotList,
                    last = last,
                    opensAtStartup = opensAtStartup,
                    onChoose = ::fly,
                    onOpenStartupSettings = { DjiPilot.openSettings(this) },
                    onSettings = {
                        refreshNetwork()
                        showSettings = true
                    },
                    syncStatus = syncStatus,
                )
            }
        }
    }

    /**
     * Uploads run once the controller is paired and may read the logs. Without
     * access, the onboarding screen explains Android's question before asking.
     */
    private fun startUploads() {
        hasLogAccess = checkSelfPermission(Manifest.permission.READ_EXTERNAL_STORAGE) ==
            PackageManager.PERMISSION_GRANTED
        logAccessRefused = !hasLogAccess &&
            getSharedPreferences("onboarding", MODE_PRIVATE).getBoolean("asked_for_logs", false)
        if (paired && hasLogAccess) UploadService.start(this)
    }

    private fun refreshNetwork() {
        canSeeNetworkNames = sync.canSeeNetworkNames()
        currentNetwork = sync.currentNetwork()
    }

    /**
     * Unpairs on the server, and forgets the token here whatever happens, so
     * the controller shows a new pairing code. If the server can't be reached,
     * an admin can still remove the pairing there.
     */
    private fun unpair() {
        lifecycleScope.launch {
            try {
                withContext(Dispatchers.IO) { server.unpair() }
            } catch (_: IOException) {
                // Forgotten here anyway; see above.
            }
            showSettings = false
            paired = false
        }
    }

    override fun onDestroy() {
        syncListener?.let { SyncStatus.stopListening(this, it) }
        super.onDestroy()
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setUp(intent)
    }

    // Again on every return, e.g. from the settings or from DJI Pilot 2.
    override fun onResume() {
        super.onResume()
        opensAtStartup = DjiPilot.opensAtStartup(this)
        refreshNetwork()
        startUploads()
        pilotList = pilots.current
        last = pilots.last()
        if (paired) refreshPilots()
    }

    /**
     * Downloads the pilot list; offline, the stored one stays. A refused token
     * means the controller was unpaired: back to pairing.
     */
    private fun refreshPilots() {
        lifecycleScope.launch {
            val downloaded = try {
                withContext(Dispatchers.IO) { server.pilots() }
            } catch (_: IOException) {
                return@launch
            }
            if (downloaded == null) {
                paired = false
                return@launch
            }
            pilots.current = downloaded
            pilotList = downloaded
            last = pilots.last()
        }
    }

    /**
     * The download page's link, fleetranger://setup?server=https://…, tells the
     * app which install it belongs to. A different install means pairing again.
     */
    private fun setUp(intent: Intent?) {
        val link: Uri = intent?.data ?: return
        if (link.scheme != "fleetranger" || link.host != "setup") return
        val url = link.getQueryParameter("server")?.trimEnd('/') ?: return
        if (!url.startsWith("https://") && !(BuildConfig.DEBUG && url.startsWith("http://"))) return
        if (url != server.url) {
            server.url = url
            serverUrl = url
            paired = false
        }
    }

    private fun fly(pilot: Pilot?) {
        pilots.choose(pilot)
        if (!DjiPilot.open(this)) {
            Toast.makeText(this, "DJI Pilot 2 isn’t installed.", Toast.LENGTH_LONG).show()
        }
    }
}
