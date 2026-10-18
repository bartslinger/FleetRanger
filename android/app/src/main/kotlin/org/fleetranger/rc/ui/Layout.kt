package org.fleetranger.rc.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsPressedAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp

// Larger building blocks: the top bar of a secondary screen, vertical tabs,
// contained lists and the informational callout.

/** A screen's top bar: a ghost "Back" with an arrow at the left, then the title. */
@Composable
fun TopBar(title: String, onBack: () -> Unit) {
    Row(Modifier.fillMaxWidth().height(64.dp), verticalAlignment = Alignment.CenterVertically) {
        val interaction = remember { MutableInteractionSource() }
        val pressed by interaction.collectIsPressedAsState()
        Row(
            Modifier
                .height(64.dp)
                .background(if (pressed) Colors.layerActive01 else Colors.background)
                .clickable(interaction, indication = null, role = Role.Button, onClick = onBack)
                .padding(horizontal = Spacing.s05),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(Spacing.s03),
        ) {
            Icon(CarbonIcons.ArrowLeft, Colors.interactive)
            BasicText("Back", style = Type.bodyCompact02.copy(color = Colors.interactive))
        }
        BasicText(
            title,
            style = Type.heading03.copy(color = Colors.textPrimary),
            modifier = Modifier.padding(start = Spacing.s05).semantics { heading() },
        )
    }
}

/**
 * Carbon's vertical tabs, for the parts of one screen: the tabs at the left,
 * the chosen tab's panel at the right, both on layer-01. One tab is always
 * selected; tabs are 64dp high for gloves.
 */
@Composable
fun VerticalTabs(
    tabs: List<String>,
    selected: Int,
    onSelect: (Int) -> Unit,
    modifier: Modifier = Modifier,
    panel: @Composable ColumnScope.() -> Unit,
) {
    Row(modifier.background(Colors.layer01)) {
        Column(Modifier.width(192.dp).fillMaxHeight()) {
            tabs.forEachIndexed { index, label ->
                val isSelected = index == selected
                Row(
                    Modifier
                        .fillMaxWidth()
                        .height(64.dp)
                        .selectable(
                            selected = isSelected,
                            interactionSource = remember { MutableInteractionSource() },
                            indication = null,
                            role = Role.Tab,
                            onClick = { onSelect(index) },
                        )
                        .drawBehind {
                            // Carbon: a 3dp bar marks the selected tab; a
                            // subtle line separates the others.
                            if (isSelected) {
                                drawRect(Colors.borderInteractive, size = size.copy(width = 3.dp.toPx()))
                            } else {
                                drawLine(
                                    Colors.borderSubtle01,
                                    Offset(0f, size.height - 0.5f),
                                    Offset(size.width, size.height - 0.5f),
                                    1.dp.toPx(),
                                )
                            }
                        }
                        .padding(horizontal = Spacing.s05),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    BasicText(
                        label,
                        style = Type.bodyCompact02.copy(
                            color = if (isSelected) Colors.textPrimary else Colors.textSecondary,
                            fontWeight = if (isSelected) FontWeight.SemiBold else FontWeight.Normal,
                        ),
                    )
                }
            }
        }
        Column(
            Modifier.weight(1f).fillMaxHeight().padding(Spacing.s05),
            content = panel,
        )
    }
}

/**
 * Carbon's contained list: a title, then rows 64dp high (extra large) with a
 * subtle line between them. The title is indented like the rows' text.
 */
@Composable
fun ContainedList(title: String, modifier: Modifier = Modifier, rows: @Composable ColumnScope.() -> Unit) {
    Column(modifier.fillMaxWidth()) {
        BasicText(
            title,
            style = Type.headingCompact01.copy(color = Colors.textPrimary),
            modifier = Modifier
                .padding(start = Spacing.s05, top = Spacing.s03, bottom = Spacing.s03)
                .semantics { heading() },
        )
        Column(Modifier.drawBehind { topLine() }, content = rows)
    }
}

/** A contained list's row: its text at the left, an optional action at the right. */
@Composable
fun ListRow(text: String, secondary: Boolean = false, action: @Composable RowScope.() -> Unit = {}) {
    Row(
        Modifier
            .fillMaxWidth()
            .height(64.dp)
            .drawBehind { bottomLine() }
            .padding(start = Spacing.s05),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        BasicText(
            text,
            style = Type.bodyCompact02.copy(color = if (secondary) Colors.textSecondary else Colors.textPrimary),
            modifier = Modifier.weight(1f),
        )
        action()
    }
}

/** Carbon's ghost icon button, 64dp square; `label` is for screen readers. */
@Composable
fun IconButton(icon: ImageVector, label: String, onClick: () -> Unit) {
    val interaction = remember { MutableInteractionSource() }
    val pressed by interaction.collectIsPressedAsState()
    Box(
        Modifier
            .size(64.dp)
            .background(if (pressed) Colors.layerActive01 else Colors.layer01)
            .clickable(interaction, indication = null, role = Role.Button, onClick = onClick)
            .semantics { contentDescription = label },
        contentAlignment = Alignment.Center,
    ) {
        Icon(icon, Colors.iconPrimary)
    }
}

/**
 * Carbon's callout, informational: a notification that stays until its cause
 * is gone, with one action.
 */
@Composable
fun InfoCallout(title: String, text: String, action: String, onAction: () -> Unit) {
    Row(
        Modifier
            .fillMaxWidth()
            .height(IntrinsicSize.Min)
            .background(Colors.notificationBackgroundInfo)
            .border(1.dp, Colors.supportInfo.copy(alpha = 0.4f)),
    ) {
        Box(Modifier.width(3.dp).fillMaxHeight().background(Colors.supportInfo))
        InformationIcon(Modifier.padding(start = Spacing.s05, top = Spacing.s04 + 1.dp))
        Column(
            Modifier.weight(1f).padding(horizontal = Spacing.s05, vertical = Spacing.s04),
            verticalArrangement = Arrangement.spacedBy(Spacing.s03),
        ) {
            BasicText(title, style = Type.headingCompact02.copy(color = Colors.textPrimary))
            BasicText(text, style = Type.bodyCompact02.copy(color = Colors.textPrimary))
            Button(action, onAction, kind = ButtonKind.Tertiary)
        }
    }
}

private fun androidx.compose.ui.graphics.drawscope.DrawScope.topLine() =
    drawLine(Colors.borderSubtle01, Offset(0f, 0.5f), Offset(size.width, 0.5f), 1.dp.toPx())

private fun androidx.compose.ui.graphics.drawscope.DrawScope.bottomLine() =
    drawLine(
        Colors.borderSubtle01,
        Offset(0f, size.height - 0.5f),
        Offset(size.width, size.height - 0.5f),
        1.dp.toPx(),
    )
