package org.fleetranger.rc

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardCapitalization
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import java.io.IOException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.fleetranger.rc.ui.Button
import org.fleetranger.rc.ui.ButtonKind
import org.fleetranger.rc.ui.Colors
import org.fleetranger.rc.ui.InlineLoading
import org.fleetranger.rc.ui.InlineNotification
import org.fleetranger.rc.ui.LabelledCode
import org.fleetranger.rc.ui.NotificationKind
import org.fleetranger.rc.ui.Spacing
import org.fleetranger.rc.ui.TextInput
import org.fleetranger.rc.ui.Type

/** The characters of a pairing code: no 0/O or 1/I, as on the server. */
private const val CODE_ALPHABET = "23456789ABCDEFGHJKLMNPQRSTUVWXYZ"
private const val CODE_LENGTH = 8

/** What the pairing form shows besides its fields. */
data class PairingForm(
    val address: String = "",
    val addressError: String? = null,
    val code: String = "",
    val codeError: String? = null,
    /** Not about a field, such as FleetRanger being out of reach. */
    val problem: String? = null,
    val busy: Boolean = false,
)

/**
 * Pairs the controller with a code someone created in the web app: the pilot
 * types it here, and the controller pairs at once, nobody logging in. See
 * docs/pairing.md. Afterwards it says which team the controller joined, with
 * a way to unpair straight away if that's wrong.
 *
 * Without a known server (an APK installed some other way), the address is
 * asked for too; it's kept only if FleetRanger answers there.
 */
@Composable
fun Pairing(server: Server, serverUrl: String?, onServer: (String?) -> Unit, onPaired: () -> Unit) {
    val scope = rememberCoroutineScope()
    var form by remember { mutableStateOf(PairingForm()) }
    // The team the controller joined (empty for none); null until it's paired.
    var pairedWith by rememberSaveable { mutableStateOf<String?>(null) }
    var unpairing by remember { mutableStateOf(false) }

    fun pair() {
        val typedUrl = if (serverUrl == null) serverAddress(form.address) else null
        if (serverUrl == null && typedUrl == null) {
            form = form.copy(addressError = "Enter an address such as example.fleetranger.org.")
            return
        }
        val code = normalize(form.code)
        if (code.length != CODE_LENGTH || code.any { it !in CODE_ALPHABET }) {
            form = form.copy(codeError = "Enter the 8 characters of the code, as the web app shows it.")
            return
        }
        // What went wrong last time may not any more.
        form = form.copy(busy = true, addressError = null, codeError = null, problem = null)
        typedUrl?.let(onServer)
        scope.launch {
            form = try {
                val team = withContext(Dispatchers.IO) { server.pair(code) }
                pairedWith = team.orEmpty()
                PairingForm()
            } catch (refused: Server.Refused) {
                form.copy(busy = false, codeError = refused.message)
            } catch (_: IOException) {
                if (typedUrl != null) {
                    onServer(null)
                    form.copy(
                        busy = false,
                        addressError = "FleetRanger didn’t answer at this address. " +
                            "Check the address and the Wi-Fi connection.",
                    )
                } else {
                    form.copy(
                        busy = false,
                        problem = "Can’t reach FleetRanger. Check the Wi-Fi connection and try again.",
                    )
                }
            }
        }
    }

    fun unpair() {
        unpairing = true
        scope.launch {
            // The token is forgotten here whatever happens; an admin can
            // remove a pairing the server still has.
            val reached = try {
                withContext(Dispatchers.IO) { server.unpair() }
                true
            } catch (_: IOException) {
                false
            }
            unpairing = false
            pairedWith = null
            form = PairingForm(
                problem = if (reached) null else "Unpaired here, but FleetRanger couldn’t be reached. " +
                    "Ask an admin to unpair this controller in Equipment.",
            )
        }
    }

    PairingScreen(
        form = form,
        needsAddress = serverUrl == null,
        pairedWith = pairedWith,
        unpairing = unpairing,
        serial = remember { Serial.read() },
        onAddressChange = { form = form.copy(address = it, addressError = null) },
        onCodeChange = { form = form.copy(code = it.uppercase(), codeError = null) },
        onPair = ::pair,
        onContinue = onPaired,
        onUnpair = ::unpair,
    )
}

@Composable
fun PairingScreen(
    form: PairingForm,
    needsAddress: Boolean,
    pairedWith: String?,
    serial: String,
    unpairing: Boolean = false,
    onAddressChange: (String) -> Unit = {},
    onCodeChange: (String) -> Unit = {},
    onPair: () -> Unit = {},
    onContinue: () -> Unit = {},
    onUnpair: () -> Unit = {},
) {
    Column(
        Modifier
            .fillMaxSize()
            .background(Colors.background)
            .verticalScroll(rememberScrollState())
            .padding(horizontal = Spacing.s07, vertical = Spacing.s06),
        verticalArrangement = Arrangement.spacedBy(Spacing.s05),
    ) {
        BasicText(
            "Pair this controller",
            style = Type.heading04.copy(color = Colors.textPrimary),
            modifier = Modifier.semantics { heading() },
        )
        val width = Modifier.widthIn(max = 480.dp)
        if (pairedWith != null) {
            InlineNotification(
                NotificationKind.Success,
                title = "Paired",
                text = if (pairedWith.isEmpty()) {
                    "This controller is paired, without a team. An admin can choose its team."
                } else {
                    "This controller is paired with $pairedWith."
                },
                modifier = width,
            )
            if (unpairing) {
                Box(Modifier.height(64.dp), contentAlignment = Alignment.CenterStart) {
                    InlineLoading("Unpairing…")
                }
            } else {
                Row(horizontalArrangement = Arrangement.spacedBy(Spacing.s01)) {
                    Button("Continue", onContinue)
                    Button("Unpair", onUnpair, kind = ButtonKind.Secondary)
                }
            }
        } else {
            if (needsAddress) {
                TextInput(
                    label = "Server address",
                    value = form.address,
                    onValueChange = onAddressChange,
                    modifier = width,
                    helper = "The address you open FleetRanger at in a browser, such as example.fleetranger.org.",
                    invalid = form.addressError,
                    enabled = !form.busy,
                    keyboardOptions = KeyboardOptions(
                        autoCorrectEnabled = false,
                        keyboardType = KeyboardType.Uri,
                        imeAction = ImeAction.Next,
                    ),
                )
            }
            TextInput(
                label = "Pairing code",
                value = form.code,
                onValueChange = onCodeChange,
                modifier = width,
                helper = "8 letters and digits, from Pair a controller in FleetRanger’s web app.",
                invalid = form.codeError,
                enabled = !form.busy,
                keyboardOptions = KeyboardOptions(
                    capitalization = KeyboardCapitalization.Characters,
                    autoCorrectEnabled = false,
                    keyboardType = KeyboardType.Ascii,
                    imeAction = ImeAction.Done,
                ),
                onDone = onPair,
            )
            form.problem?.let {
                InlineNotification(NotificationKind.Error, title = "Not paired", text = it, modifier = width)
            }
            if (form.busy) {
                Box(Modifier.height(64.dp), contentAlignment = Alignment.CenterStart) {
                    InlineLoading("Pairing…")
                }
            } else {
                Button("Pair", onPair)
            }
        }
        LabelledCode("Serial number", serial)
    }
}

/** A code as typed: any case, with or without the dash or spaces. */
private fun normalize(code: String) = code.filter { it.isLetterOrDigit() }.uppercase()

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

@Preview(widthDp = 768, heightDp = 432)
@Composable
private fun PairingScreenPreview() {
    PairingScreen(PairingForm(code = "KDW4-9TXR"), needsAddress = false, pairedWith = null, serial = "TESTRC0001")
}

@Preview(widthDp = 768, heightDp = 432)
@Composable
private fun PairedPreview() {
    PairingScreen(PairingForm(), needsAddress = false, pairedWith = "Test team", serial = "TESTRC0001")
}
