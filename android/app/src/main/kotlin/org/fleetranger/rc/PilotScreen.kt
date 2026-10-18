package org.fleetranger.rc

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import org.fleetranger.rc.ui.ClickableTile
import org.fleetranger.rc.ui.Colors
import org.fleetranger.rc.ui.Spacing
import org.fleetranger.rc.ui.Type
import org.fleetranger.rc.ui.WarningNotification

/** Every tile the same height, whether or not it has a second line. */
private val TILE_HEIGHT = 88.dp

/**
 * "Who's flying?": one tile per pilot, the last pilot first, and "Not in the
 * list". Choosing one hands over to DJI Pilot 2.
 */
@Composable
fun PilotScreen(
    pilots: List<Pilot>,
    last: Pilot?,
    opensAtStartup: Boolean,
    onChoose: (Pilot?) -> Unit,
    onOpenSettings: () -> Unit,
) {
    val ordered = listOfNotNull(last) + pilots.filter { it != last }
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
                action = "Open settings",
                onAction = onOpenSettings,
            )
        }
        BasicText(
            "Who’s flying?",
            style = Type.heading04.copy(color = Colors.textPrimary),
            modifier = Modifier.semantics { heading() },
        )
        // Tiles 2dp apart, as in Carbon's tile groups.
        LazyVerticalGrid(
            columns = GridCells.Adaptive(minSize = 200.dp),
            horizontalArrangement = Arrangement.spacedBy(Spacing.s01),
            verticalArrangement = Arrangement.spacedBy(Spacing.s01),
        ) {
            items(ordered, key = { it.id }) { pilot ->
                ClickableTile(
                    title = pilot.name,
                    detail = if (pilot == last) "Flew last" else null,
                    onClick = { onChoose(pilot) },
                    modifier = Modifier.height(TILE_HEIGHT),
                )
            }
            item(key = "not-in-the-list") {
                ClickableTile(
                    title = "Not in the list",
                    detail = "Set the pilot later in the logbook",
                    onClick = { onChoose(null) },
                    modifier = Modifier.height(TILE_HEIGHT),
                )
            }
        }
    }
}

@Preview(widthDp = 768, heightDp = 432)
@Composable
private fun PilotScreenPreview() {
    val pilots = listOf(Pilot("a", "Test pilot A"), Pilot("b", "Test pilot B"))
    PilotScreen(pilots, last = pilots[1], opensAtStartup = false, onChoose = {}, onOpenSettings = {})
}
