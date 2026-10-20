package org.fleetranger.rc

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.GridItemSpan
import androidx.compose.foundation.lazy.grid.LazyGridScope
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.lazy.grid.rememberLazyGridState
import androidx.compose.ui.graphics.Brush
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import org.fleetranger.rc.ui.Button
import org.fleetranger.rc.ui.ButtonKind
import org.fleetranger.rc.ui.CarbonIcons
import org.fleetranger.rc.ui.ClickableTile
import org.fleetranger.rc.ui.Colors
import org.fleetranger.rc.ui.Icon
import org.fleetranger.rc.ui.InformationIcon
import org.fleetranger.rc.ui.Spacing
import org.fleetranger.rc.ui.Type
import org.fleetranger.rc.ui.WarningIcon
import org.fleetranger.rc.ui.WarningNotification

/** Every tile in the list the same height, whether or not it has a second line. */
private val TILE_HEIGHT = 104.dp

/** "Not in the list", as chosen from the list, apart from any pilot's key. */
private const val NOT_LISTED = "not-listed"

/**
 * "Who's flying?": the pilot's tile opens DJI Pilot 2, and says so. The last
 * pilot is there already, so the same pilot again is one tap. Anyone else is
 * chosen through "Other pilot" and the list, which goes back to this screen
 * with them on the tile, so a different pilot or team is a step more, on
 * purpose (user's decision, 2026-10-20). Without a last pilot (a new
 * controller, or that pilot was removed) the list shows straight away, and
 * choosing goes to this screen too. A choice in the list counts at once,
 * also if DJI Pilot 2 is opened some other way (user's request, 2026-10-20).
 */
@Composable
fun PilotScreen(
    list: PilotList,
    last: Pilot?,
    opensAtStartup: Boolean,
    onChoose: (Pilot?) -> Unit,
    onSelect: (Pilot?) -> Unit,
    onOpenStartupSettings: () -> Unit,
    onSettings: () -> Unit,
    syncStatus: SyncStatus? = null,
    update: Server.AppVersion? = null,
    onUpdate: () -> Unit = {},
) {
    var choosing by rememberSaveable { mutableStateOf(false) }
    // Who was chosen in the list, by key; null for the last pilot.
    var chosenKey by rememberSaveable { mutableStateOf<String?>(null) }
    // The pilot on the tile: the one chosen, or else the last. A pilot in a
    // Pick, so "Not in the list" (no pilot) can be on the tile too.
    val current: Pick? = when (val key = chosenKey) {
        NOT_LISTED -> Pick(null)
        null -> last?.let(::Pick)
        else -> list.pilots.find { it.key == key }?.let(::Pick) ?: last?.let(::Pick)
    }
    val settingsButton: @Composable () -> Unit = {
        Button("Settings", onSettings, kind = ButtonKind.Ghost)
    }
    BackHandler(enabled = choosing && current != null) { choosing = false }
    val choose: (Pilot?) -> Unit = { pilot ->
        onSelect(pilot)
        chosenKey = pilot?.key ?: NOT_LISTED
        choosing = false
    }

    Column(
        Modifier
            .fillMaxSize()
            .background(Colors.background)
            // The screen is only ~390dp high: 16dp at the top and bottom and
            // between blocks, so the startup warning and the sync line leave
            // room for the pilot tiles.
            .padding(horizontal = Spacing.s07, vertical = Spacing.s05),
        verticalArrangement = Arrangement.spacedBy(Spacing.s05),
    ) {
        if (!opensAtStartup) {
            WarningNotification(
                title = "FleetRanger doesn’t open at startup",
                text = "Choose it under Apps & notifications › Auto start application selection.",
                action = "Open Android settings",
                onAction = onOpenStartupSettings,
            )
        }
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(Spacing.s05)) {
            if (current != null && !choosing) {
                Heading("Who’s flying?", settingsButton)
                CurrentPilot(
                    current,
                    onFly = {
                        onChoose(current.pilot)
                        chosenKey = null
                    },
                    onOther = { choosing = true },
                )
            } else if (current != null) {
                Heading("Other pilot") {
                    Button("Back", { choosing = false }, kind = ButtonKind.Ghost)
                }
                // The one on the first screen is there already.
                PilotList(
                    list.copy(pilots = list.pilots.filter { it != current.pilot }),
                    choose,
                    showNotListed = current.pilot != null,
                )
            } else {
                Heading("Who’s flying?", settingsButton)
                PilotList(list, choose)
            }
        }
        // Not on "Other pilot", which needs the room for the list; the first
        // screen shows it. An update shares the row, so it takes no height.
        val choosingOther = current != null && choosing
        if ((syncStatus != null || update != null) && !choosingOther) {
            Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                if (syncStatus != null) SyncLine(syncStatus, onSettings)
                Spacer(Modifier.weight(1f))
                if (update != null) UpdateLine(update, onUpdate)
            }
        }
    }
}

/** A newer release is on the server: opens the settings' App tab to update. */
@Composable
private fun UpdateLine(update: Server.AppVersion, onUpdate: () -> Unit) {
    Row(
        Modifier
            .height(48.dp)
            .clickable(onClickLabel = "Update the app", onClick = onUpdate)
            .padding(horizontal = Spacing.s05),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(Spacing.s03),
    ) {
        InformationIcon()
        BasicText(
            "Update available: version ${update.name}",
            style = Type.bodyCompact02.copy(color = Colors.interactive),
        )
        Icon(CarbonIcons.ArrowRight, Colors.interactive)
    }
}

/**
 * The upload status, as a ghost button that opens the settings: logs that
 * aren't uploading get a warning icon, so it isn't missed.
 */
@Composable
private fun SyncLine(status: SyncStatus, onSettings: () -> Unit) {
    // 48dp, Carbon's large size: still a good target, and 16dp less height.
    Row(
        Modifier
            .height(48.dp)
            .clickable(onClickLabel = "Open settings", onClick = onSettings)
            .padding(horizontal = Spacing.s05),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(Spacing.s03),
    ) {
        if (status.problem) WarningIcon()
        BasicText(
            "Sync: ${status.summary()}",
            style = Type.bodyCompact02.copy(color = Colors.interactive),
        )
        Icon(CarbonIcons.ArrowRight, Colors.interactive)
    }
}

/** Who flies: a pilot, or none for "Not in the list". */
private data class Pick(val pilot: Pilot?)

/** The pilot on the tile, in one tap to DJI Pilot 2; or another one. */
@Composable
private fun CurrentPilot(current: Pick, onFly: () -> Unit, onOther: () -> Unit) {
    val pilot = current.pilot
    // Carbon's wide gutter: a gloved tap mustn't hit the wrong one.
    // 144dp when there's room, at least 96dp with the startup warning above.
    Row(
        Modifier.fillMaxWidth().heightIn(min = 96.dp, max = 144.dp),
        horizontalArrangement = Arrangement.spacedBy(Spacing.s07),
    ) {
        ClickableTile(
            title = pilot?.name ?: "Not in the list",
            detail = pilot?.teamName ?: "Set the pilot later in the logbook",
            action = "Open DJI Pilot 2",
            icon = CarbonIcons.Launch,
            onClick = onFly,
            // Carbon's heading-05 is regular; semibold on purpose, to read outdoors.
            titleStyle = Type.heading05.copy(color = Colors.textPrimary, fontWeight = FontWeight.SemiBold),
            modifier = Modifier.weight(2f).fillMaxHeight(),
        )
        ClickableTile(
            title = "Other pilot",
            detail = "Choose from the list",
            onClick = onOther,
            modifier = Modifier.weight(1f).fillMaxHeight(),
        )
    }
}

/**
 * Everyone: the controller's team, then other teams, and "Not in the list"
 * unless it's on the first screen already. Choosing only chooses.
 */
@Composable
private fun PilotList(list: PilotList, onChoose: (Pilot?) -> Unit, showNotListed: Boolean = true) {
    val (team, others) = list.pilots.partition { list.teamId == null || it.teamId == list.teamId }
    val state = rememberLazyGridState()
    Box {
    // Carbon's narrow gutter between tiles, for gloves.
    LazyVerticalGrid(
        columns = GridCells.Adaptive(minSize = 200.dp),
        state = state,
        horizontalArrangement = Arrangement.spacedBy(Spacing.s05),
        verticalArrangement = Arrangement.spacedBy(Spacing.s05),
    ) {
        // The team shows on every tile: the flights belong to the one chosen.
        items(team, key = { it.key }) { pilot ->
            PilotTile(pilot, detail = pilot.teamName, onChoose)
        }
        if (showNotListed) {
            item(key = "not-in-the-list") {
                ClickableTile(
                    title = "Not in the list",
                    detail = "Set the pilot later in the logbook",
                    onClick = { onChoose(null) },
                    modifier = Modifier.height(TILE_HEIGHT),
                )
            }
        }
        if (others.isNotEmpty()) {
            sectionHeading("Other teams")
            items(others, key = { it.key }) { pilot ->
                PilotTile(pilot, detail = pilot.teamName, onChoose)
            }
        }
    }
    // While more pilots are below, the list fades out at the bottom, so it's
    // clear it scrolls.
    if (state.canScrollForward) {
        Box(
            Modifier
                .align(Alignment.BottomCenter)
                .fillMaxWidth()
                .height(48.dp)
                .background(Brush.verticalGradient(listOf(Colors.background.copy(alpha = 0f), Colors.background))),
        )
    }
    }
}

private fun LazyGridScope.sectionHeading(text: String) {
    item(key = "heading-$text", span = { GridItemSpan(maxLineSpan) }) {
        BasicText(
            text,
            style = Type.headingCompact02.copy(color = Colors.textPrimary),
            modifier = Modifier
                .padding(top = Spacing.s06, bottom = Spacing.s03)
                .semantics { heading() },
        )
    }
}

/**
 * The screen's heading, with an optional action at the right. Always as tall
 * as a button, so the heading stays put when the action comes and goes.
 */
@Composable
private fun Heading(text: String, action: @Composable () -> Unit = {}) {
    Row(
        Modifier.fillMaxWidth().height(64.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        BasicText(
            text,
            style = Type.heading04.copy(color = Colors.textPrimary),
            modifier = Modifier.semantics { heading() },
        )
        action()
    }
}

@Composable
private fun PilotTile(pilot: Pilot, detail: String?, onChoose: (Pilot?) -> Unit) {
    ClickableTile(
        title = pilot.name,
        detail = detail,
        onClick = { onChoose(pilot) },
        modifier = Modifier.height(TILE_HEIGHT),
    )
}

private val previewPilots = listOf(
    Pilot("a", "Test pilot A", "north", "Test team North"),
    Pilot("b", "Test pilot B", "north", "Test team North"),
    Pilot("c", "Test pilot C", "south", "Test team South"),
)

@Preview(widthDp = 768, heightDp = 432)
@Composable
private fun LastPilotPreview() {
    PilotScreen(PilotList("north", previewPilots), previewPilots[1], true, {}, {}, {}, {})
}

@Preview(widthDp = 768, heightDp = 432)
@Composable
private fun PilotListPreview() {
    PilotScreen(PilotList("north", previewPilots), last = null, true, {}, {}, {}, {})
}
