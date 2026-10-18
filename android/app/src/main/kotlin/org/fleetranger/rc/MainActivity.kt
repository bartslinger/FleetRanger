package org.fleetranger.rc

import android.os.Bundle
import android.widget.Toast
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue

/**
 * The app's only screen: who's flying, then DJI Pilot 2. The controller opens
 * it at startup once it is chosen as the startup app.
 */
class MainActivity : ComponentActivity() {
    private lateinit var pilots: Pilots
    private var opensAtStartup by mutableStateOf(true)
    private var last by mutableStateOf<Pilot?>(null)

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        pilots = Pilots(this)
        setContent {
            PilotScreen(
                pilots = pilots.all,
                last = last,
                opensAtStartup = opensAtStartup,
                onChoose = ::fly,
                onOpenSettings = { DjiPilot.openSettings(this) },
            )
        }
    }

    // Again on every return, e.g. from the settings or from DJI Pilot 2.
    override fun onResume() {
        super.onResume()
        opensAtStartup = DjiPilot.opensAtStartup(this)
        last = pilots.last()
    }

    private fun fly(pilot: Pilot?) {
        pilots.choose(pilot)
        if (!DjiPilot.open(this)) {
            Toast.makeText(this, "DJI Pilot 2 isn’t installed.", Toast.LENGTH_LONG).show()
        }
    }
}
