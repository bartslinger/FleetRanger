package org.fleetranger.rc

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import java.io.IOException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.withContext
import org.fleetranger.rc.ui.Button
import org.fleetranger.rc.ui.Colors
import org.fleetranger.rc.ui.InlineLoading
import org.fleetranger.rc.ui.LabelledCode
import org.fleetranger.rc.ui.PlexMono
import org.fleetranger.rc.ui.QrCode
import org.fleetranger.rc.ui.Spacing
import org.fleetranger.rc.ui.TextInput
import org.fleetranger.rc.ui.Type
import org.fleetranger.rc.ui.WarningNotification

/** How often to ask whether a user approved the code; asking keeps it valid. */
private const val CHECK_EVERY_MS = 3_000L
/** How long to wait before trying an unreachable server again. */
private const val RETRY_AFTER_MS = 10_000L
/** How long a code is shown before pairing stops, so a forgotten controller
 * doesn't keep a pairing notification up in the web app. */
private const val GIVE_UP_AFTER_MS = 60 * 60_000L

/** What the pairing screen shows. */
sealed interface PairingState {
    /**
     * The app doesn't know its server yet: the download page's link sets it
     * up, or someone types it. `error` says why the last address didn't work.
     */
    data class NoServer(val error: String? = null) : PairingState
    data object Starting : PairingState
    data class Unreachable(val server: String) : PairingState
    data class Waiting(val server: String, val code: String) : PairingState
    /** Stopped after an hour without approval, until someone asks for a new code. */
    data object Stopped : PairingState
}

/**
 * Pairs the controller: asks the server for a code, shows it with a QR code
 * for a user to approve, and checks until they have. The code stays the same
 * while the screen is open; if the server forgot it (the controller was
 * offline for a while), a new code replaces it.
 *
 * A typed address is checked by asking it for a code: only FleetRanger
 * answers with one. If it doesn't, the address is forgotten again.
 */
@Composable
fun Pairing(server: Server, serverUrl: String?, onServer: (String?) -> Unit, onPaired: () -> Unit) {
    var state by remember { mutableStateOf<PairingState>(PairingState.Starting) }
    // Raised by "Show a new code", to start pairing again.
    var attempt by remember { mutableIntStateOf(0) }
    // A typed address that hasn't answered yet, and why the last one failed.
    var typed by remember { mutableStateOf(false) }
    var addressError by remember { mutableStateOf<String?>(null) }
    // Kept here, so the field still has it when an address didn't work.
    var address by rememberSaveable { mutableStateOf("") }
    LaunchedEffect(serverUrl, attempt) {
        if (serverUrl == null) {
            state = PairingState.NoServer(addressError)
            return@LaunchedEffect
        }
        val started = System.currentTimeMillis()
        while (System.currentTimeMillis() - started < GIVE_UP_AFTER_MS) {
            val code = try {
                withContext(Dispatchers.IO) { server.startPairing() }
            } catch (_: IOException) {
                if (typed) {
                    typed = false
                    addressError = "FleetRanger didn’t answer at this address. " +
                        "Check the address and the Wi-Fi connection."
                    onServer(null)
                    return@LaunchedEffect
                }
                state = PairingState.Unreachable(serverUrl)
                delay(RETRY_AFTER_MS)
                continue
            }
            typed = false
            state = PairingState.Waiting(serverUrl, code)
            while (System.currentTimeMillis() - started < GIVE_UP_AFTER_MS) {
                delay(CHECK_EVERY_MS)
                val check = try {
                    withContext(Dispatchers.IO) { server.check() }
                } catch (_: IOException) {
                    // Offline for a moment: keep the code; if the server forgot
                    // it meanwhile, the next check says so.
                    Server.Check.Waiting
                }
                if (check == Server.Check.Paired) {
                    onPaired()
                    return@LaunchedEffect
                }
                if (check == Server.Check.Unknown) break
            }
        }
        state = PairingState.Stopped
    }
    PairingScreen(
        state,
        serial = remember { Serial.read() },
        onRestart = { attempt++ },
        address = address,
        onAddressChange = {
            address = it
            // A new address hasn't failed yet.
            if (addressError != null) {
                addressError = null
                state = PairingState.NoServer()
            }
        },
        onConnect = {
            val url = serverAddress(address)
            if (url == null) {
                addressError = "Enter an address such as example.fleetranger.org"
                state = PairingState.NoServer(addressError)
            } else {
                addressError = null
                typed = true
                state = PairingState.Starting
                onServer(url)
            }
        },
    )
}

@Composable
fun PairingScreen(
    state: PairingState,
    serial: String,
    onRestart: () -> Unit = {},
    address: String = "",
    onAddressChange: (String) -> Unit = {},
    onConnect: () -> Unit = {},
) {
    Row(
        Modifier
            .fillMaxSize()
            .background(Colors.background)
            .padding(horizontal = Spacing.s07, vertical = Spacing.s06),
        horizontalArrangement = Arrangement.spacedBy(Spacing.s07),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(Spacing.s05)) {
            BasicText(
                "Pair this controller",
                style = Type.heading04.copy(color = Colors.textPrimary),
                modifier = Modifier.semantics { heading() },
            )
            when (state) {
                is PairingState.NoServer -> AddressForm(address, onAddressChange, state.error, onConnect)
                PairingState.Starting -> InlineLoading("Asking for a code…")
                PairingState.Stopped -> {
                    Text("Pairing stopped after an hour without approval.")
                    Button("Show a new code", onRestart)
                }
                is PairingState.Unreachable -> WarningNotification(
                    title = "Can’t reach FleetRanger",
                    text = "Check the Wi-Fi connection. Trying ${state.server} again shortly.",
                )
                is PairingState.Waiting -> {
                    Text("Scan the QR code with your phone, or go to this address and enter the code:")
                    BasicText(pageUrl(state.server), style = Type.code02.copy(color = Colors.textPrimary))
                    // Monospaced, so 0 and O, 1 and I can't be confused.
                    BasicText(
                        formatCode(state.code),
                        style = Type.heading06.copy(color = Colors.textPrimary, fontFamily = PlexMono),
                    )
                    InlineLoading("Waiting for approval…")
                }
            }
            LabelledCode("Serial number", serial)
        }
        if (state is PairingState.Waiting) {
            QrCode(
                text = "${pageUrl(state.server)}?code=${formatCode(state.code)}",
                size = 240.dp,
                description = "QR code to approve this controller",
            )
        }
    }
}

@Composable
private fun Text(text: String, secondary: Boolean = false) {
    BasicText(
        text,
        style = Type.bodyCompact02.copy(
            color = if (secondary) Colors.textSecondary else Colors.textPrimary,
        ),
    )
}

/** Where this controller's FleetRanger is, typed: what people use in the browser. */
@Composable
private fun AddressForm(
    address: String,
    onAddressChange: (String) -> Unit,
    error: String?,
    onConnect: () -> Unit,
) {
    TextInput(
        label = "Server address",
        value = address,
        onValueChange = onAddressChange,
        modifier = Modifier.widthIn(max = 480.dp),
        placeholder = "example.fleetranger.org",
        helper = "The address you open FleetRanger at in a browser",
        invalid = error,
        keyboardOptions = KeyboardOptions(
            autoCorrectEnabled = false,
            keyboardType = KeyboardType.Uri,
            imeAction = ImeAction.Go,
        ),
        onDone = onConnect,
    )
    Button("Connect", onConnect)
}

/**
 * "example.fleetranger.org" as "https://example.fleetranger.org"; null if it can't be
 * an address. Debug builds also take http://, for a dev server.
 */
private fun serverAddress(input: String): String? {
    val text = input.trim().trimEnd('/')
    if (text.isEmpty() || text.any { it.isWhitespace() }) return null
    val url = if ("://" in text) text else "https://$text"
    if (!url.startsWith("https://") && !(BuildConfig.DEBUG && url.startsWith("http://"))) return null
    return url
}

private fun pageUrl(server: String) = "$server/pair"

/** KDW49TXR as KDW4-9TXR, easier to read and type. */
private fun formatCode(code: String) = code.chunked(4).joinToString("-")

@Preview(widthDp = 768, heightDp = 432)
@Composable
private fun PairingScreenPreview() {
    PairingScreen(PairingState.Waiting("https://fleet.example.org", "KDW49TXR"), serial = "TESTRC0001")
}
