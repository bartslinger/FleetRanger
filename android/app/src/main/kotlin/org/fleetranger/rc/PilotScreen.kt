package org.fleetranger.rc

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
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.GridItemSpan
import androidx.compose.foundation.lazy.grid.LazyGridScope
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
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
import androidx.compose.ui.unit.sp
import org.fleetranger.rc.ui.Button
import org.fleetranger.rc.ui.ButtonKind
import org.fleetranger.rc.ui.ClickableTile
import org.fleetranger.rc.ui.Colors
import org.fleetranger.rc.ui.Spacing
import org.fleetranger.rc.ui.Type
import org.fleetranger.rc.ui.WarningNotification

/** Every tile in the list the same height, whether or not it has a second line. */
private val TILE_HEIGHT = 88.dp

/**
 * "Who's flying?": the last pilot is one tap; anyone else is two, through
 * "Other pilot" and the list. Choosing hands over to DJI Pilot 2. Without a
 * last pilot (a new controller, or that pilot was removed) the list shows
 * straight away.
 */
@Composable
fun PilotScreen(
    list: PilotList,
    last: Pilot?,
    opensAtStartup: Boolean,
    onChoose: (Pilot?) -> Unit,
    onOpenStartupSettings: () -> Unit,
    onSettings: () -> Unit,
    syncStatus: SyncStatus? = null,
) {
    var choosing by rememberSaveable { mutableStateOf(false) }
    val settingsButton: @Composable () -> Unit = {
        Button("Settings", onSettings, kind = ButtonKind.Ghost)
    }
    BackHandler(enabled = choosing && last != null) { choosing = false }

    Column(
        Modifier
            .fillMaxSize()
            .background(Colors.background)
            .padding(horizontal = Spacing.s07, vertical = Spacing.s06),
        verticalArrangement = Arrangement.spacedBy(Spacing.s06),
    ) {
        if (!opensAtStartup) {
            WarningNotification(
                title = "FleetRanger doesn’t open at startup",
                text = "Choose it under Apps & notifications › Auto start application selection.",
                action = "Open Android settings",
                onAction = onOpenStartupSettings,
            )
        }
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(Spacing.s06)) {
            if (last != null && !choosing) {
                Heading("Who’s flying?", settingsButton)
                LastPilot(last, onFly = { onChoose(last) }, onOther = { choosing = true })
            } else if (last != null) {
                Heading("Other pilot") {
                    Button("Back", { choosing = false }, kind = ButtonKind.Ghost)
                }
                // The last pilot is on the first screen already.
                PilotList(list.copy(pilots = list.pilots.filter { it != last }), onChoose)
            } else {
                Heading("Who’s flying?", settingsButton)
                PilotList(list, onChoose)
            }
        }
        if (syncStatus != null) SyncLine(syncStatus, onSettings)
    }
}

/** The upload status in one line; tapping it opens the settings. */
@Composable
private fun SyncLine(status: SyncStatus, onSettings: () -> Unit) {
    BasicText(
        "Sync: ${status.summary()}",
        style = Type.label01.copy(color = Colors.textSecondary),
        modifier = Modifier
            .fillMaxWidth()
            .clickable(onClickLabel = "Open settings", onClick = onSettings)
            .padding(vertical = Spacing.s03),
    )
}

/** The usual case: the same pilot again, in one tap. */
@Composable
private fun LastPilot(last: Pilot, onFly: () -> Unit, onOther: () -> Unit) {
    // Carbon's wide gutter: a gloved tap mustn't hit the wrong one.
    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(Spacing.s07)) {
        ClickableTile(
            title = last.name,
            detail = "Flew last",
            onClick = onFly,
            titleStyle = Type.heading04.copy(
                color = Colors.textPrimary,
                fontWeight = FontWeight.SemiBold,
                fontSize = 32.sp,
                lineHeight = 40.sp,
            ),
            modifier = Modifier.weight(2f).height(144.dp),
        )
        ClickableTile(
            title = "Other pilot",
            detail = "Choose from the list",
            onClick = onOther,
            modifier = Modifier.weight(1f).height(144.dp),
        )
    }
}

/** Everyone: the controller's team, then other teams, and "Not in the list". */
@Composable
private fun PilotList(list: PilotList, onChoose: (Pilot?) -> Unit) {
    val (team, others) = list.pilots.partition { list.teamId == null || it.teamId == list.teamId }
    // Carbon's narrow gutter between tiles, for gloves.
    LazyVerticalGrid(
        columns = GridCells.Adaptive(minSize = 200.dp),
        horizontalArrangement = Arrangement.spacedBy(Spacing.s05),
        verticalArrangement = Arrangement.spacedBy(Spacing.s05),
    ) {
        items(team, key = { it.id }) { pilot ->
            PilotTile(pilot, detail = null, onChoose)
        }
        item(key = "not-in-the-list") {
            ClickableTile(
                title = "Not in the list",
                detail = "Set the pilot later in the logbook",
                onClick = { onChoose(null) },
                modifier = Modifier.height(TILE_HEIGHT),
            )
        }
        if (others.isNotEmpty()) {
            sectionHeading("Other teams")
            items(others, key = { it.id }) { pilot ->
                PilotTile(pilot, detail = pilot.teamName, onChoose)
            }
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
    PilotScreen(PilotList("north", previewPilots), previewPilots[1], true, {}, {}, {})
}

@Preview(widthDp = 768, heightDp = 432)
@Composable
private fun PilotListPreview() {
    PilotScreen(PilotList("north", previewPilots), last = null, true, {}, {}, {})
}
