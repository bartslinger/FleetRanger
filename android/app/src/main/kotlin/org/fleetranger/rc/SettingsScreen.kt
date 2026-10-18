package org.fleetranger.rc

import android.app.DatePickerDialog
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneId
import java.time.format.DateTimeFormatter
import java.util.Locale
import org.fleetranger.rc.ui.Button
import org.fleetranger.rc.ui.ButtonKind
import org.fleetranger.rc.ui.CarbonIcons
import org.fleetranger.rc.ui.Colors
import org.fleetranger.rc.ui.DangerDialog
import org.fleetranger.rc.ui.Icon
import org.fleetranger.rc.ui.LabelledCode
import org.fleetranger.rc.ui.RadioButton
import org.fleetranger.rc.ui.Spacing
import org.fleetranger.rc.ui.Type

/** As in the web app, e.g. "12 Oct 2026". */
private val DATE = DateTimeFormatter.ofPattern("d MMM yyyy", Locale.ENGLISH)
private val TIME = DateTimeFormatter.ofPattern("d MMM yyyy, HH:mm", Locale.ENGLISH)

/**
 * The controller's settings: when logs sync, and its pairing. Anyone holding
 * the controller may change them.
 *
 * - `currentNetwork`: the Wi-Fi network the controller is on, if Android
 *   reveals its name.
 * - `canSeeNetworkNames`: false until the location permission is granted.
 */
@Composable
fun SettingsScreen(
    settings: SyncSettings,
    serverUrl: String?,
    currentNetwork: String?,
    canSeeNetworkNames: Boolean,
    onAllowNetworkNames: () -> Unit,
    onUnpair: () -> Unit,
    onBack: () -> Unit,
    syncStatus: SyncStatus? = null,
) {
    BackHandler(onBack = onBack)
    var networks by remember { mutableStateOf(settings.networks) }
    var syncFrom by remember { mutableStateOf(settings.syncFrom) }
    var confirmingUnpair by remember { mutableStateOf(false) }
    val context = LocalContext.current

    fun pickDate() {
        val initial = syncFrom ?: LocalDate.now()
        DatePickerDialog(
            context,
            { _, year, month, day ->
                val date = LocalDate.of(year, month + 1, day)
                settings.syncFrom = date
                syncFrom = date
            },
            initial.year,
            initial.monthValue - 1,
            initial.dayOfMonth,
        ).show()
    }

    Column(
        Modifier
            .fillMaxSize()
            .background(Colors.background)
            .verticalScroll(rememberScrollState())
            .padding(horizontal = Spacing.s07, vertical = Spacing.s06),
    ) {
        Row(
            Modifier.fillMaxWidth().height(64.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            BasicText(
                "Settings",
                style = Type.heading04.copy(color = Colors.textPrimary),
                modifier = Modifier.semantics { heading() },
            )
            Button("Back", onBack, kind = ButtonKind.Ghost)
        }

        Column(Modifier.widthIn(max = 560.dp)) {
            if (syncStatus != null) {
                Section("Sync")
                Paragraph(syncStatus.summary())
                Paragraph(
                    when {
                        syncStatus.uploaded == 0 -> "No logs uploaded yet"
                        else -> "${syncStatus.uploaded} ${if (syncStatus.uploaded == 1) "log" else "logs"} " +
                            "uploaded, the last at " +
                            TIME.format(Instant.ofEpochMilli(syncStatus.lastUploadAt).atZone(ZoneId.systemDefault()))
                    },
                    secondary = true,
                )
            }
            Section("Wi-Fi networks for syncing")
            Paragraph(
                "Flight logs upload only over these networks, and never over one Android " +
                    "reports as metered, such as most phone hotspots.",
            )
            if (networks.isEmpty()) {
                Paragraph("None yet.", secondary = true)
            }
            networks.forEach { network ->
                Row(
                    Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    BasicText(network, style = Type.bodyCompact02.copy(color = Colors.textPrimary))
                    Button("Remove", {
                        networks = networks - network
                        settings.networks = networks
                    }, kind = ButtonKind.Ghost)
                }
            }
            Column(Modifier.padding(top = Spacing.s05)) {
                when {
                    !canSeeNetworkNames -> {
                        Paragraph(
                            "To see which network the controller is on, FleetRanger needs the " +
                                "location permission. It doesn’t use the location.",
                            secondary = true,
                        )
                        Button("Allow", onAllowNetworkNames, kind = ButtonKind.Tertiary)
                    }
                    currentNetwork == null -> Paragraph("Not connected to Wi-Fi.", secondary = true)
                    currentNetwork in networks ->
                        Paragraph("Connected to $currentNetwork.", secondary = true)
                    else -> {
                        Paragraph("Connected to $currentNetwork.", secondary = true)
                        Button("Add this network", {
                            networks = networks + currentNetwork
                            settings.networks = networks
                        }, kind = ButtonKind.Tertiary)
                    }
                }
            }

            Section("Sync logs from")
            RadioButton(
                "All logs",
                selected = syncFrom == null,
                onSelect = {
                    settings.syncFrom = null
                    syncFrom = null
                },
            )
            RadioButton("From a date", selected = syncFrom != null, onSelect = ::pickDate)
            syncFrom?.let { date ->
                // The chosen date as a field of its own, indented to the labels.
                DateField("Start date", DATE.format(date), ::pickDate, Modifier.padding(start = 28.dp))
            }

            Section("Pairing")
            Paragraph("Paired with")
            BasicText(serverUrl ?: "no server", style = Type.code02.copy(color = Colors.textPrimary))
            LabelledCode("Serial number", Serial.read(), Modifier.padding(top = Spacing.s03))
            // Rare, and never the point of this screen: not a primary danger button.
            Button(
                "Unpair controller",
                { confirmingUnpair = true },
                kind = ButtonKind.DangerTertiary,
                modifier = Modifier.padding(top = Spacing.s06),
            )
        }
    }

    if (confirmingUnpair) {
        DangerDialog(
            title = "Unpair controller",
            text = "The controller stops uploading flight logs and shows a new pairing code. " +
                "Logs already uploaded stay in FleetRanger.",
            action = "Unpair",
            onConfirm = {
                confirmingUnpair = false
                onUnpair()
            },
            onDismiss = { confirmingUnpair = false },
        )
    }
}

/** A section of the settings, 32dp below the previous one. */
@Composable
private fun Section(text: String) {
    BasicText(
        text,
        style = Type.headingCompact02.copy(color = Colors.textPrimary),
        modifier = Modifier
            .padding(top = Spacing.s07, bottom = Spacing.s03)
            .semantics { heading() },
    )
}

@Composable
private fun Paragraph(text: String, secondary: Boolean = false) {
    BasicText(
        text,
        style = Type.bodyCompact02.copy(
            color = if (secondary) Colors.textSecondary else Colors.textPrimary,
        ),
        modifier = Modifier.padding(bottom = Spacing.s03),
    )
}

/**
 * A field showing a date, which opens Android's own date picker: Carbon's
 * date input (label, value, calendar icon), 64dp high for gloves.
 */
@Composable
private fun DateField(label: String, value: String, onPick: () -> Unit, modifier: Modifier = Modifier) {
    Column(modifier.padding(top = Spacing.s03)) {
        BasicText(label, style = Type.label02.copy(color = Colors.textSecondary))
        Row(
            Modifier
                .padding(top = Spacing.s03)
                .widthIn(min = 288.dp)
                .height(64.dp)
                .background(Colors.layer01)
                .drawBehind {
                    drawLine(
                        Colors.borderStrong01,
                        Offset(0f, size.height - 0.5f),
                        Offset(size.width, size.height - 0.5f),
                        strokeWidth = 1.dp.toPx(),
                    )
                }
                .clickable(onClickLabel = "Change $label", onClick = onPick)
                .padding(horizontal = Spacing.s05),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            BasicText(value, style = Type.bodyCompact02.copy(color = Colors.textPrimary))
            Icon(CarbonIcons.Calendar, Colors.iconPrimary)
        }
    }
}
