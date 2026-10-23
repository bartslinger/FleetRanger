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
    /** "Who's flying?": the pilot's tile and "Other pilot". */
    First,
    Pilots,
    /** "Which group are you flying for?": the group's tile and "Other group". */
    Group,
    Groups,
}

/**
 * "Who's flying?": the pilot's tile opens DJI Pilot 2, and says so. The last
 * pilot is there already with their group, so the same again is one tap.
 * For a pilot who may fly for more than one group, the tile leads on to
 * "Which group are you flying for?", built the same way: their last group's
 * tile opens DJI Pilot 2, and "Other group" the list of their groups (user's
 * decision, 2026-10-23: two taps then, and changing the group doesn't go
 * through "Other pilot"). Anyone else is chosen through "Other pilot" and the
 * list, which goes back to this screen with them on the tile, or on to the
 * group screen, so a different pilot or group is a step more, on purpose
 * (user's decision, 2026-10-20). Without a last pilot (a new controller, or
 * that pilot was removed) the list shows straight away. A choice counts at
 * once, also if DJI Pilot 2 is opened some other way (user's request,
 * 2026-10-20).
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
    // The choice on the tiles: the one made, or else the last. A choice in a
    // Pick, so "Not in the list" (no pilot) can be on the tile too.
    val current: Pick? = when (val key = chosenKey) {
        NOT_LISTED -> Pick(null)
        null -> last?.let(::Pick)
        else -> {
            val pilotId = key.substringBefore(':')
            val groupId = key.substringAfter(':', "").ifEmpty { null }
            list.pilots.find { it.id == pilotId }?.let { list.choiceFor(it, groupId) }?.let(::Pick)
                ?: last?.let(::Pick)
        }
    }
    val choice = current?.choice
    val choosesGroup = choice != null && list.choosesGroup(choice.pilot)
    // Without a choice on the tile, the list is the first thing to show.
    val shown = when {
        current == null -> Step.Pilots
        (step == Step.Group || step == Step.Groups) && !choosesGroup -> Step.First
        else -> step
    }
    val settingsButton: @Composable () -> Unit = {
        Button("Settings", onSettings, kind = ButtonKind.Ghost)
    }
    val select: (Choice?) -> Unit = { selected ->
        onSelect(selected)
        chosenKey = selected?.key ?: NOT_LISTED
    }
    val choosePilot: (Pilot?) -> Unit = { pilot ->
        if (pilot == null) {
            select(null)
            step = Step.First
        } else {
            // The same pilot again keeps their group.
            select(list.choiceFor(pilot, choice?.takeIf { it.pilot.id == pilot.id }?.groupId))
            step = if (list.choosesGroup(pilot)) Step.Group else Step.First
        }
    }
    val fly: () -> Unit = {
        onChoose(choice)
        chosenKey = null
    }
    val back: () -> Unit = {
        step = if (shown == Step.Groups) Step.Group else Step.First
    }
    BackHandler(enabled = shown != Step.First && current != null) { back() }
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
                    val pilot = choice?.pilot
                    ChoiceTiles(
                        title = pilot?.name ?: "Not in the list",
                        // A pilot's tile names only the pilot (user's decision).
                        detail = if (pilot == null) "Set the pilot later in the logbook" else null,
                        // Into the group screen, or straight to DJI Pilot 2.
                        launches = !choosesGroup,
                        onMain = { if (choosesGroup) step = Step.Group else fly() },
                        other = "Other pilot",
                        onOther = { step = Step.Pilots },
                    )
                }
                Step.Pilots -> {
                    if (current == null) {
                        Heading("Who’s flying?", settingsButton)
                    } else {
                        Heading("Other pilot", backButton)
                    }
                    PilotList(list, choosePilot, showNotListed = current == null || choice != null)
                }
                Step.Group -> {
                    Heading("Which group are you flying for?", backButton)
                    ChoiceTiles(
                        title = list.group(choice!!.groupId)?.name.orEmpty(),
                        detail = null,
                        launches = true,
                        onMain = fly,
                        other = "Other group",
                        onOther = { step = Step.Groups },
                    )
                }
                Step.Groups -> {
                    Heading("Other group", backButton)
                    GroupList(list, choice!!) {
                        select(Choice(choice.pilot, it))
                        step = Step.Group
                    }
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
 * The choice so far in a big tile, and another one beside it: the pilot and
 * "Other pilot", or the group and "Other group". A tile that opens DJI Pilot
 * 2 says so with the Launch icon; one that leads to the next screen has the
 * arrow and no action text (carbon-expert's suggestion).
 */
@Composable
private fun ChoiceTiles(
    title: String,
    detail: String?,
    launches: Boolean,
    onMain: () -> Unit,
    other: String,
    onOther: () -> Unit,
) {
    // Carbon's wide gutter: a gloved tap mustn't hit the wrong one.
    // 144dp when there's room, at least 96dp with the startup warning above.
    Row(
        Modifier.fillMaxWidth().heightIn(min = 96.dp, max = 144.dp),
        horizontalArrangement = Arrangement.spacedBy(Spacing.s07),
    ) {
        ClickableTile(
            title = title,
            detail = detail,
            action = if (launches) "Open DJI Pilot 2" else null,
            icon = if (launches) CarbonIcons.Launch else CarbonIcons.ArrowRight,
            onClick = onMain,
            // Carbon's heading-05 is regular; semibold on purpose, to read outdoors.
            titleStyle = Type.heading05.copy(color = Colors.textPrimary, fontWeight = FontWeight.SemiBold),
            modifier = Modifier.weight(2f).fillMaxHeight(),
        )
        ClickableTile(
            title = other,
            detail = "Choose from the list",
            onClick = onOther,
            modifier = Modifier.weight(1f).fillMaxHeight(),
        )
    }
}

/**
 * Everyone who flies, each once: the controller's group's pilots first, then
 * the others, and "Not in the list" unless it's on the first screen already.
 * Choosing a pilot goes on to the group screen when they may fly for more
 * than one group.
 */
@Composable
private fun PilotList(list: PilotList, onChoose: (Pilot?) -> Unit, showNotListed: Boolean = true) {
    val (own, others) = list.pilots.partition { list.controllerGroup == null || list.controllerGroup in it.groups }
    TileGrid {
        items(own, key = { it.id }) { pilot ->
            Tile(pilot.name, detail = null) { onChoose(pilot) }
        }
        if (showNotListed) {
            item(key = "not-in-the-list") {
                Tile("Not in the list", "Set the pilot later in the logbook") { onChoose(null) }
            }
        }
        if (others.isNotEmpty()) {
            sectionHeading("Other pilots")
            items(others, key = { it.id }) { pilot ->
                Tile(pilot.name, detail = null) { onChoose(pilot) }
            }
        }
    }
}

/**
 * The other groups the pilot may fly for, by name: their own first, then
 * the rest under "Other groups", as the pilot list puts the controller's
 * group first. The group already on the tile isn't repeated.
 */
@Composable
private fun GroupList(list: PilotList, choice: Choice, onChoose: (String) -> Unit) {
    val own = choice.pilot.groups.filter { it != choice.groupId }
    val others = choice.pilot.otherGroups.filter { it != choice.groupId }
    TileGrid {
        items(own, key = { it }) { id ->
            Tile(list.group(id)?.name.orEmpty(), detail = null) { onChoose(id) }
        }
        if (others.isNotEmpty()) {
            // Only apart from their own groups; without any, it's one list.
            if (choice.pilot.groups.isNotEmpty()) sectionHeading("Other groups")
            items(others, key = { it }) { id ->
                Tile(list.group(id)?.name.orEmpty(), detail = null) { onChoose(id) }
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
        // A long heading wraps rather than pushing the action off the screen.
        BasicText(
            text,
            style = Type.heading04.copy(color = Colors.textPrimary),
            modifier = Modifier.weight(1f, fill = false).semantics { heading() },
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
