package org.fleetranger.rc

import android.app.DatePickerDialog
import android.content.Intent
import android.provider.Settings
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.snapshotFlow
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import java.io.IOException
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneId
import java.time.format.DateTimeFormatter
import java.util.Locale
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.fleetranger.rc.ui.Button
import org.fleetranger.rc.ui.ButtonKind
import org.fleetranger.rc.ui.CarbonIcons
import org.fleetranger.rc.ui.Colors
import org.fleetranger.rc.ui.ContainedList
import org.fleetranger.rc.ui.DangerDialog
import org.fleetranger.rc.ui.Icon
import org.fleetranger.rc.ui.IconButton
import org.fleetranger.rc.ui.InfoCallout
import org.fleetranger.rc.ui.InlineLoading
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

/** The settings' tabs, in their order. */
enum class SettingsTab(val label: String) {
    Sync("Sync"),
    Networks("Networks"),
    Pairing("Pairing"),
    App("App"),
}

/**
 * The controller's settings: when logs sync, its pairing, and the app's
 * version and updates. Anyone holding the controller may change them. Groups
 * that nobody compares, so Carbon's vertical tabs.
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
    onShowLogs: () -> Unit,
    onBack: () -> Unit,
    syncStatus: SyncStatus?,
    initialTab: SettingsTab,
    server: Server,
    updates: Updates,
    update: Server.AppVersion?,
) {
    BackHandler(onBack = onBack)
    var tab by rememberSaveable { mutableStateOf(initialTab) }
    var uploadOver by remember { mutableStateOf(settings.uploadOver) }
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
            tabs = SettingsTab.entries.map { it.label },
            selected = tab.ordinal,
            onSelect = { tab = SettingsTab.entries[it] },
            modifier = Modifier
                .weight(1f)
                .padding(start = Spacing.s05, end = Spacing.s05, bottom = Spacing.s05),
        ) {
            // The panel keeps its height; long content scrolls inside it, and
            // fades out at the bottom while there's more, so it's clear it scrolls.
            val scroll = remember(tab) { ScrollState(0) }
            // Choosing approved networks scrolls their lists into view.
            var revealLists by remember { mutableStateOf(false) }
            LaunchedEffect(revealLists) {
                if (!revealLists) return@LaunchedEffect
                snapshotFlow { scroll.maxValue }.first { it > 0 && it < Int.MAX_VALUE }
                scroll.animateScrollTo(scroll.maxValue)
                revealLists = false
            }
            Box(Modifier.weight(1f).fillMaxWidth()) {
            Column(Modifier.verticalScroll(scroll)) {
                when (tab) {
                    SettingsTab.Sync -> SyncTab(
                        syncStatus,
                        syncFrom,
                        onAllLogs = {
                            settings.syncFrom = null
                            syncFrom = null
                            onChanged()
                        },
                        onPickDate = ::pickDate,
                        onAddNetwork = { tab = SettingsTab.Networks },
                        onShowLogs = onShowLogs,
                    )
                    SettingsTab.Networks -> NetworksTab(
                        uploadOver,
                        onUploadOver = {
                            uploadOver = it
                            settings.uploadOver = it
                            onChanged()
                            if (it == SyncSettings.UploadOver.ApprovedWifi) revealLists = true
                        },
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
                    SettingsTab.Pairing -> PairingTab(serverUrl, onUnpair = { confirmingUnpair = true })
                    SettingsTab.App -> AppTab(server, updates, update)
                }
            }
            if (scroll.canScrollForward) {
                Box(
                    Modifier
                        .align(Alignment.BottomCenter)
                        .fillMaxWidth()
                        .height(48.dp)
                        .background(Brush.verticalGradient(listOf(Colors.layer01.copy(alpha = 0f), Colors.layer01))),
                )
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
    onShowLogs: () -> Unit,
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
    }
    // Carbon lets a ghost button's text line up with the text above it.
    Row(Modifier.offset(x = -Spacing.s05)) {
        if (status?.state == SyncStatus.State.WaitingForNetwork &&
            status.uploadOver == SyncSettings.UploadOver.ApprovedWifi
        ) {
            Button("Add a network", onAddNetwork, kind = ButtonKind.Ghost)
        }
        Button("Show logs", onShowLogs, kind = ButtonKind.Ghost)
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
private fun ColumnScope.NetworksTab(
    uploadOver: SyncSettings.UploadOver,
    onUploadOver: (SyncSettings.UploadOver) -> Unit,
    networks: Set<String>,
    currentNetwork: String?,
    canSeeNetworkNames: Boolean,
    onAllowNetworkNames: () -> Unit,
    onAdd: (String) -> Unit,
    onRemove: (String) -> Unit,
) {
    val context = LocalContext.current
    Label("Upload logs over")
    RadioButton(
        "Approved Wi-Fi networks",
        selected = uploadOver == SyncSettings.UploadOver.ApprovedWifi,
        onSelect = { onUploadOver(SyncSettings.UploadOver.ApprovedWifi) },
    )
    RadioButton(
        "Unmetered Wi-Fi networks",
        selected = uploadOver == SyncSettings.UploadOver.AnyWifi,
        onSelect = { onUploadOver(SyncSettings.UploadOver.AnyWifi) },
        detail = "Android doesn’t recognise every hotspot. To be sure, choose approved networks.",
    )
    RadioButton(
        "Any connection",
        selected = uploadOver == SyncSettings.UploadOver.AnyConnection,
        onSelect = { onUploadOver(SyncSettings.UploadOver.AnyConnection) },
        detail = "Mobile data too, which may cost money",
    )
    if (uploadOver != SyncSettings.UploadOver.ApprovedWifi) return
    ContainedList("Approved networks", Modifier.padding(top = Spacing.s05)) {
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
                    // Back from Android's settings, the app looks again (onResume).
                    currentNetwork == null -> ListRow("Connect to Wi-Fi to approve a network", secondary = true) {
                        Button(
                            "Wi-Fi settings",
                            { context.startActivity(Intent(Settings.ACTION_WIFI_SETTINGS)) },
                            kind = ButtonKind.Tertiary,
                        )
                    }
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

/** Where checking or updating stands, in the App tab. */
private enum class UpdateStep { Idle, Checking, UpToDate, Unreachable, Downloading, DownloadFailed }

/**
 * The app's version and its updates: checked daily by the upload service, or
 * here on request. Updating downloads the release and opens it in Android's
 * installer, which asks to confirm (the first time also to allow FleetRanger
 * to install apps) and then offers to open the new version.
 */
@Composable
private fun ColumnScope.AppTab(
    server: Server,
    updates: Updates,
    update: Server.AppVersion?,
) {
    var step by remember { mutableStateOf(UpdateStep.Idle) }
    val scope = rememberCoroutineScope()
    val context = LocalContext.current

    fun check() {
        step = UpdateStep.Checking
        scope.launch {
            step = try {
                withContext(Dispatchers.IO) { updates.check(server) }
                UpdateStep.UpToDate
            } catch (_: IOException) {
                UpdateStep.Unreachable
            }
        }
    }

    fun install() {
        step = UpdateStep.Downloading
        scope.launch {
            step = try {
                val installer = withContext(Dispatchers.IO) { updates.download(server) }
                // Android's installer takes over from here; cancelled, the update is offered again.
                context.startActivity(installer)
                UpdateStep.Idle
            } catch (_: Exception) {
                // Mostly the connection; also storage that's full.
                UpdateStep.DownloadFailed
            }
        }
    }

    Label("Version")
    Text(BuildConfig.VERSION_NAME)
    Column(Modifier.padding(top = Spacing.s06), verticalArrangement = Arrangement.spacedBy(Spacing.s03)) {
        when {
            step == UpdateStep.Checking -> InlineLoading("Checking for updates…")
            step == UpdateStep.Downloading -> InlineLoading("Downloading the update…")
            update != null -> {
                Text(
                    if (step == UpdateStep.DownloadFailed) {
                        "The update couldn’t be downloaded. Check the connection and try again."
                    } else {
                        "Version ${update.name} is available."
                    },
                )
                Button("Update", ::install, Modifier.padding(top = Spacing.s03))
            }
            else -> {
                when (step) {
                    UpdateStep.UpToDate -> Text("This is the latest version.")
                    UpdateStep.Unreachable -> Text("FleetRanger can’t be reached. Try again later.")
                    else -> Unit
                }
                Button("Check for updates", ::check, kind = ButtonKind.Tertiary)
            }
        }
    }
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
fun DateField(value: String, onPick: () -> Unit, modifier: Modifier = Modifier) {
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
