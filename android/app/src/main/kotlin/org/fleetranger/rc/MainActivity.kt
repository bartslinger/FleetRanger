package org.fleetranger.rc

import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.view.WindowManager
import android.widget.Toast
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
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

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        // Like DJI Pilot 2, the screen stays on while the app is in front.
        window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
        pilots = Pilots(this)
        server = Server(this)
        setUp(intent)
        serverUrl = server.url
        paired = server.paired
        setContent {
            if (!paired) {
                Pairing(server, serverUrl, onPaired = {
                    paired = true
                    refreshPilots()
                })
            } else {
                PilotScreen(
                    list = pilotList,
                    last = last,
                    opensAtStartup = opensAtStartup,
                    onChoose = ::fly,
                    onOpenSettings = { DjiPilot.openSettings(this) },
                )
            }
        }
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setUp(intent)
    }

    // Again on every return, e.g. from the settings or from DJI Pilot 2.
    override fun onResume() {
        super.onResume()
        opensAtStartup = DjiPilot.opensAtStartup(this)
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
