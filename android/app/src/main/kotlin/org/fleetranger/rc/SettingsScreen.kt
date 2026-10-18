package org.fleetranger.rc

import android.app.DatePickerDialog
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.platform.LocalContext
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
import org.fleetranger.rc.ui.ContainedList
import org.fleetranger.rc.ui.DangerDialog
import org.fleetranger.rc.ui.Icon
import org.fleetranger.rc.ui.IconButton
import org.fleetranger.rc.ui.InfoCallout
import org.fleetranger.rc.ui.ListRow
import org.fleetranger.rc.ui.PlexMono
import org.fleetranger.rc.ui.RadioButton
import org.fleetranger.rc.ui.Spacing
import org.fleetranger.rc.ui.TopBar
import org.fleetranger.rc.ui.Type
import org.fleetranger.rc.ui.VerticalTabs
import org.fleetranger.rc.ui.WarningIcon

/** As in the web app, e.g. "12 Oct 2026". */
private val DATE = DateTimeFormatter.ofPattern("d MMM yyyy", Locale.ENGLISH)
private val TIME = DateTimeFormatter.ofPattern("d MMM yyyy, HH:mm", Locale.ENGLISH)

private const val SYNC = 0
private const val WIFI = 1
private const val PAIRING = 2

/**
 * The controller's settings: when logs sync, and its pairing. Anyone holding
 * the controller may change them. Three groups that nobody compares, so
 * Carbon's vertical tabs, each fitting the screen without scrolling.
 *
 * - `currentNetwork`: the Wi-Fi network the controller is on, if Android
 *   reveals its name.
 * - `canSeeNetworkNames`: false until the location permission is granted.
 * - `onChanged`: after a change to the networks or the date, so the uploads
 *   look again straight away and the count of waiting logs follows.
 */
@Composable
fun SettingsScreen(
    settings: SyncSettings,
    serverUrl: String?,
    currentNetwork: String?,
    canSeeNetworkNames: Boolean,
    onAllowNetworkNames: () -> Unit,
    onUnpair: () -> Unit,
    onChanged: () -> Unit,
    onBack: () -> Unit,
    syncStatus: SyncStatus? = null,
) {
    BackHandler(onBack = onBack)
    var tab by rememberSaveable { mutableIntStateOf(SYNC) }
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
                onChanged()
            },
            initial.year,
            initial.monthValue - 1,
            initial.dayOfMonth,
        ).show()
    }

    Column(Modifier.fillMaxSize().background(Colors.background)) {
        TopBar("Settings", onBack)
        VerticalTabs(
            tabs = listOf("Sync", "Wi-Fi networks", "Pairing"),
            selected = tab,
            onSelect = { tab = it },
            modifier = Modifier
                .weight(1f)
                .padding(start = Spacing.s05, end = Spacing.s05, bottom = Spacing.s05),
        ) {
            // The panel keeps its height; long content scrolls inside it.
            Column(Modifier.verticalScroll(rememberScrollState())) {
                when (tab) {
                    SYNC -> SyncTab(
                        syncStatus,
                        syncFrom,
                        onAllLogs = {
                            settings.syncFrom = null
                            syncFrom = null
                            onChanged()
                        },
                        onPickDate = ::pickDate,
                        onAddNetwork = { tab = WIFI },
                    )
                    WIFI -> WifiTab(
                        networks,
                        currentNetwork,
                        canSeeNetworkNames,
                        onAllowNetworkNames,
                        onAdd = { network ->
                            networks = networks + network
                            settings.networks = networks
                            onChanged()
                        },
                        onRemove = { network ->
                            networks = networks - network
                            settings.networks = networks
                            onChanged()
                        },
                    )
                    PAIRING -> PairingTab(serverUrl, onUnpair = { confirmingUnpair = true })
                }
            }
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

@Composable
private fun ColumnScope.SyncTab(
    status: SyncStatus?,
    syncFrom: LocalDate?,
    onAllLogs: () -> Unit,
    onPickDate: () -> Unit,
    onAddNetwork: () -> Unit,
) {
    if (status != null) {
        Row(horizontalArrangement = Arrangement.spacedBy(Spacing.s03)) {
            // The icon's 20dp centred on the text's first 22dp line.
            if (status.problem) WarningIcon(Modifier.padding(top = 1.dp))
            Text(status.summary())
        }
        Text(
            when {
                status.uploaded == 0 -> "No logs uploaded yet"
                else -> "${status.uploaded} ${if (status.uploaded == 1) "log" else "logs"} uploaded, " +
                    "the last at " +
                    TIME.format(Instant.ofEpochMilli(status.lastUploadAt).atZone(ZoneId.systemDefault()))
            },
            small = true,
        )
        if (status.state == SyncStatus.State.WaitingForNetwork) {
            // Carbon lets a ghost button's text line up with the text above it.
            Button("Add a network", onAddNetwork, Modifier.offset(x = -Spacing.s05), kind = ButtonKind.Ghost)
        }
    }
    Label("Sync logs from", Modifier.padding(top = Spacing.s06))
    // The radio buttons and, once chosen, the date beside "From a date".
    RadioButton("All logs", selected = syncFrom == null, onSelect = onAllLogs)
    Row(verticalAlignment = Alignment.CenterVertically) {
        RadioButton("From a date", selected = syncFrom != null, onSelect = onPickDate)
        if (syncFrom != null) {
            DateField(DATE.format(syncFrom), onPickDate, Modifier.padding(start = Spacing.s06))
        }
    }
}

@Composable
private fun ColumnScope.WifiTab(
    networks: Set<String>,
    currentNetwork: String?,
    canSeeNetworkNames: Boolean,
    onAllowNetworkNames: () -> Unit,
    onAdd: (String) -> Unit,
    onRemove: (String) -> Unit,
) {
    Text("Logs upload only over these networks, never over metered ones such as phone hotspots")
    ContainedList("Approved networks") {
        if (networks.isEmpty()) ListRow("No approved networks yet", secondary = true)
        networks.forEach { network ->
            ListRow(network) {
                IconButton(CarbonIcons.TrashCan, "Remove $network") { onRemove(network) }
            }
        }
    }
    Column(Modifier.padding(top = Spacing.s05)) {
        if (!canSeeNetworkNames) {
            InfoCallout(
                title = "Network name needed",
                text = "Android shows the network name only with location access. " +
                    "FleetRanger doesn’t use the location.",
                action = "Allow",
                onAction = onAllowNetworkNames,
            )
        } else {
            ContainedList("Current network") {
                when {
                    currentNetwork == null -> ListRow("Not connected to Wi-Fi", secondary = true)
                    currentNetwork in networks -> ListRow(currentNetwork) {
                        BasicText(
                            "Approved",
                            style = Type.bodyCompact02.copy(color = Colors.textSecondary),
                            modifier = Modifier.padding(end = Spacing.s05),
                        )
                    }
                    else -> ListRow(currentNetwork) {
                        Button("Add", { onAdd(currentNetwork) }, kind = ButtonKind.Tertiary)
                    }
                }
            }
        }
    }
}

@Composable
private fun ColumnScope.PairingTab(serverUrl: String?, onUnpair: () -> Unit) {
    Label("Server")
    BasicText(serverUrl ?: "None", style = Type.bodyCompact02.copy(color = Colors.textPrimary))
    Label("Serial number", Modifier.padding(top = Spacing.s05))
    BasicText(Serial.read(), style = Type.bodyCompact02.copy(color = Colors.textPrimary, fontFamily = PlexMono))
    // Rare, and never the point of this screen: not a primary danger button.
    Button(
        "Unpair controller",
        onUnpair,
        kind = ButtonKind.DangerTertiary,
        modifier = Modifier.padding(top = Spacing.s07),
    )
}

/** A label above a value or group, 14sp secondary. */
@Composable
private fun Label(text: String, modifier: Modifier = Modifier) {
    BasicText(
        text,
        style = Type.label02.copy(color = Colors.textSecondary),
        modifier = modifier.padding(bottom = Spacing.s02),
    )
}

@Composable
private fun Text(text: String, small: Boolean = false) {
    BasicText(
        text,
        style = if (small) {
            Type.label02.copy(color = Colors.textSecondary)
        } else {
            Type.bodyCompact02.copy(color = Colors.textPrimary)
        },
        modifier = Modifier.padding(bottom = Spacing.s03),
    )
}

/**
 * The chosen date as a field that opens Android's own date picker: Carbon's
 * date input (value and calendar icon) on the white panel, so in field-02.
 */
@Composable
private fun DateField(value: String, onPick: () -> Unit, modifier: Modifier = Modifier) {
    Row(
        modifier
            .widthIn(min = 200.dp)
            .height(64.dp)
            .background(Colors.background)
            .drawBehind {
                drawLine(
                    Colors.borderStrong01,
                    Offset(0f, size.height - 0.5f),
                    Offset(size.width, size.height - 0.5f),
                    strokeWidth = 1.dp.toPx(),
                )
            }
            .clickable(onClickLabel = "Change the start date", onClick = onPick)
            .padding(horizontal = Spacing.s05),
        horizontalArrangement = Arrangement.spacedBy(Spacing.s05),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        BasicText(value, style = Type.bodyCompact02.copy(color = Colors.textPrimary))
        Icon(CarbonIcons.Calendar, Colors.iconPrimary)
    }
}
