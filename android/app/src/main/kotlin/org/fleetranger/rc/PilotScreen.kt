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

/** "Not in the list", as chosen from the list, apart from any choice's key. */
private const val NOT_LISTED = "not-listed"

/** Which part of the screen shows. */
private enum class Step {
    /** The pilot's tile and "Other pilot". */
    First,
    Pilots,
    /** The group step: the pilot's own groups, with "Other group". */
    Groups,
    OtherGroups,
}

/**
 * "Who's flying?": the pilot's tile opens DJI Pilot 2, and says so. The last
 * pilot is there already with their group, so the same again is one tap.
 * Anyone else is chosen through "Other pilot" and the list, then the group
 * step when there's more than one group to choose (see "Flights and pilots"
 * in docs/architecture.md), which goes back to this screen with them on the
 * tile, so a different pilot or group is a step more, on purpose (user's
 * decision, 2026-10-20). Without a last pilot (a new controller, or that
 * pilot was removed) the list shows straight away, and choosing goes to this
 * screen too. A choice counts at once, also if DJI Pilot 2 is opened some
 * other way (user's request, 2026-10-20).
 */
@Composable
fun PilotScreen(
    list: PilotList,
    last: Choice?,
    opensAtStartup: Boolean,
    onChoose: (Choice?) -> Unit,
    onSelect: (Choice?) -> Unit,
    onOpenStartupSettings: () -> Unit,
    onSettings: () -> Unit,
    syncStatus: SyncStatus? = null,
    update: Server.AppVersion? = null,
    onUpdate: () -> Unit = {},
) {
    var step by rememberSaveable { mutableStateOf(Step.First) }
    // Who was chosen, by key; null for the last choice.
    var chosenKey by rememberSaveable { mutableStateOf<String?>(null) }
    // The pilot whose group is being chosen.
    var stepPilotId by rememberSaveable { mutableStateOf<String?>(null) }
    // The choice on the tile: the one made, or else the last. A choice in a
    // Pick, so "Not in the list" (no pilot) can be on the tile too.
    val current: Pick? = when (val key = chosenKey) {
        NOT_LISTED -> Pick(null)
        null -> last?.let(::Pick)
        else -> {
            val pilotId = key.substringBefore(':')
            val groupId = key.substringAfter(':', "").ifEmpty { null }
            list.pilots.find { it.id == pilotId }?.let { list.choice(it, groupId) }?.let(::Pick)
                ?: last?.let(::Pick)
        }
    }
    val stepPilot = list.pilots.find { it.id == stepPilotId }
    // Without a choice on the tile, the list is the first thing to show.
    val shown = when {
        step == Step.First && current == null -> Step.Pilots
        (step == Step.Groups || step == Step.OtherGroups) && stepPilot == null -> Step.Pilots
        else -> step
    }
    val settingsButton: @Composable () -> Unit = {
        Button("Settings", onSettings, kind = ButtonKind.Ghost)
    }
    val choose: (Choice?) -> Unit = { choice ->
        onSelect(choice)
        chosenKey = choice?.key ?: NOT_LISTED
        step = Step.First
    }
    val choosePilot: (Pilot?) -> Unit = { pilot ->
        if (pilot == null) {
            choose(null)
        } else {
            val choice = list.choice(pilot, null)
            if (choice != null) {
                choose(choice)
            } else {
                stepPilotId = pilot.id
                step = Step.Groups
            }
        }
    }
    val back: () -> Unit = {
        step = when (shown) {
            Step.OtherGroups -> Step.Groups
            Step.Groups -> Step.Pilots
            else -> Step.First
        }
    }
    BackHandler(enabled = shown != Step.First && !(shown == Step.Pilots && current == null)) { back() }
    val backButton: @Composable () -> Unit = {
        Button("Back", back, kind = ButtonKind.Ghost)
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
            when (shown) {
                Step.First -> {
                    Heading("Who’s flying?", settingsButton)
                    CurrentPilot(
                        current!!,
                        detail = current.choice?.let { list.group(it.groupId)?.name },
                        onFly = {
                            onChoose(current.choice)
                            chosenKey = null
                        },
                        onOther = { step = Step.Pilots },
                    )
                }
                Step.Pilots -> {
                    if (current == null) {
                        Heading("Who’s flying?", settingsButton)
                    } else {
                        Heading("Other pilot", backButton)
                    }
                    PilotList(list, choosePilot, showNotListed = current == null || current.choice != null)
                }
                Step.Groups -> {
                    val pilot = stepPilot!!
                    Heading("Which group is this flight for?", backButton)
                    GroupList(
                        list,
                        list.groupsFor(pilot),
                        onChoose = { choose(Choice(pilot, it)) },
                        // Their own groups first; the other ones on request.
                        others = if (pilot.groups.isEmpty()) 0 else pilot.otherGroups.size,
                        onOthers = { step = Step.OtherGroups },
                    )
                }
                Step.OtherGroups -> {
                    val pilot = stepPilot!!
                    Heading("Other group", backButton)
                    GroupList(list, pilot.otherGroups, onChoose = { choose(Choice(pilot, it)) })
                }
            }
        }
        // Only on the first screen, as the lists need the room. An update
        // shares the row, so it takes no height.
        if ((syncStatus != null || update != null) && shown == Step.First) {
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

/** Who flies: a pilot with their group, or none for "Not in the list". */
private data class Pick(val choice: Choice?)

/**
 * The pilot on the tile, in one tap to DJI Pilot 2; or another one. The
 * group is the tile's second line (carbon-expert's suggestion), none when
 * the organisation has no groups.
 */
@Composable
private fun CurrentPilot(current: Pick, detail: String?, onFly: () -> Unit, onOther: () -> Unit) {
    val pilot = current.choice?.pilot
    // Carbon's wide gutter: a gloved tap mustn't hit the wrong one.
    // 144dp when there's room, at least 96dp with the startup warning above.
    Row(
        Modifier.fillMaxWidth().heightIn(min = 96.dp, max = 144.dp),
        horizontalArrangement = Arrangement.spacedBy(Spacing.s07),
    ) {
        ClickableTile(
            title = pilot?.name ?: "Not in the list",
            detail = if (pilot == null) "Set the pilot later in the logbook" else detail,
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
 * Everyone who flies, each once: the controller's group's pilots first, then
 * the others, and "Not in the list" unless it's on the first screen already.
 * Each tile names the pilot's own groups. Choosing a pilot goes on to the
 * group step when there's a group to choose.
 */
@Composable
private fun PilotList(list: PilotList, onChoose: (Pilot?) -> Unit, showNotListed: Boolean = true) {
    val (own, others) = list.pilots.partition { list.controllerGroup == null || list.controllerGroup in it.groups }
    val groupNames = { pilot: Pilot ->
        pilot.groups.mapNotNull { list.group(it)?.name }.joinToString(", ").ifEmpty { null }
    }
    TileGrid {
        items(own, key = { it.id }) { pilot ->
            Tile(pilot.name, groupNames(pilot)) { onChoose(pilot) }
        }
        if (showNotListed) {
            item(key = "not-in-the-list") {
                Tile("Not in the list", "Set the pilot later in the logbook") { onChoose(null) }
            }
        }
        if (others.isNotEmpty()) {
            sectionHeading("Other pilots")
            items(others, key = { it.id }) { pilot ->
                Tile(pilot.name, groupNames(pilot)) { onChoose(pilot) }
            }
        }
    }
}

/**
 * Groups to choose from, by name, and "Other group" for `others` more,
 * which opens a list of its own (carbon-expert's advice: tiles don't reveal
 * more in place).
 */
@Composable
private fun GroupList(
    list: PilotList,
    groupIds: List<String>,
    onChoose: (String) -> Unit,
    others: Int = 0,
    onOthers: () -> Unit = {},
) {
    TileGrid {
        items(groupIds, key = { it }) { id ->
            Tile(list.group(id)?.name.orEmpty(), detail = null) { onChoose(id) }
        }
        if (others > 0) {
            item(key = "other-group") {
                Tile("Other group", if (others == 1) "1 more group" else "$others more groups", onOthers)
            }
        }
    }
}

/**
 * Tiles in a scrolling grid, with Carbon's narrow gutter between them, for
 * gloves. While more are below, the list fades out at the bottom, so it's
 * clear it scrolls.
 */
@Composable
private fun TileGrid(content: LazyGridScope.() -> Unit) {
    val state = rememberLazyGridState()
    Box {
        LazyVerticalGrid(
            columns = GridCells.Adaptive(minSize = 200.dp),
            state = state,
            horizontalArrangement = Arrangement.spacedBy(Spacing.s05),
            verticalArrangement = Arrangement.spacedBy(Spacing.s05),
            content = content,
        )
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
private fun Tile(title: String, detail: String?, onClick: () -> Unit) {
    ClickableTile(
        title = title,
        detail = detail,
        onClick = onClick,
        modifier = Modifier.height(TILE_HEIGHT),
    )
}

private val previewList = PilotList(
    controllerGroup = "north",
    groups = listOf(Group("north", "Test group North"), Group("south", "Test group South"), Group("west", "Test group West")),
    pilots = listOf(
        Pilot("a", "Test pilot A", listOf("north"), emptyList()),
        Pilot("b", "Test pilot B", listOf("north", "south"), listOf("west")),
        Pilot("c", "Test pilot C", emptyList(), listOf("north", "south", "west")),
    ),
)

@Preview(widthDp = 768, heightDp = 432)
@Composable
private fun LastPilotPreview() {
    PilotScreen(previewList, Choice(previewList.pilots[1], "south"), true, {}, {}, {}, {})
}

@Preview(widthDp = 768, heightDp = 432)
@Composable
private fun PilotListPreview() {
    PilotScreen(previewList, last = null, true, {}, {}, {}, {})
}
