package org.fleetranger.rc.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsFocusedAsState
import androidx.compose.foundation.interaction.collectIsPressedAsState
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.unit.dp

// Carbon's components, as far as the app needs them. Touch targets are at
// least 64dp, Carbon's extra large size: pilots may wear gloves.

enum class ButtonKind { Primary, Secondary, Tertiary, Ghost, Danger }

/** Carbon's button, extra large (64dp), with its text at the top left. */
@Composable
fun Button(
    text: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    kind: ButtonKind = ButtonKind.Primary,
) {
    val interaction = remember { MutableInteractionSource() }
    val pressed by interaction.collectIsPressedAsState()
    val focused by interaction.collectIsFocusedAsState()
    val background = when (kind) {
        ButtonKind.Primary -> if (pressed) Colors.buttonPrimaryActive else Colors.buttonPrimary
        ButtonKind.Secondary -> if (pressed) Colors.buttonSecondaryActive else Colors.buttonSecondary
        ButtonKind.Tertiary -> if (pressed) Colors.buttonTertiaryActive else Color.Transparent
        ButtonKind.Ghost -> if (pressed) Colors.layerActive01 else Color.Transparent
        ButtonKind.Danger -> if (pressed) Colors.buttonDangerActive else Colors.buttonDanger
    }
    val foreground = when {
        kind == ButtonKind.Ghost -> Colors.interactive
        kind == ButtonKind.Tertiary && !pressed -> Colors.interactive
        else -> Colors.textOnColor
    }
    val border = when {
        focused -> Colors.focus
        kind == ButtonKind.Tertiary -> Colors.interactive
        else -> Color.Transparent
    }
    // Carbon leaves room on the right for an icon, except on ghost buttons,
    // whose text is centred vertically.
    val ghost = kind == ButtonKind.Ghost
    Box(
        modifier
            .height(64.dp)
            .background(background)
            .border(if (focused) 2.dp else 1.dp, border)
            .clickable(interaction, indication = null, role = Role.Button, onClick = onClick)
            .padding(
                start = Spacing.s05,
                end = if (ghost) Spacing.s05 else 64.dp,
                top = if (ghost) 0.dp else Spacing.s04,
            ),
        contentAlignment = if (ghost) Alignment.CenterStart else Alignment.TopStart,
    ) {
        BasicText(text, style = Type.bodyCompact01.copy(color = foreground))
    }
}

/** Carbon's clickable tile: one large target, a title and an optional line under it. */
@Composable
fun ClickableTile(
    title: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    detail: String? = null,
    titleStyle: TextStyle = Type.headingCompact02.copy(color = Colors.textPrimary),
) {
    val interaction = remember { MutableInteractionSource() }
    val pressed by interaction.collectIsPressedAsState()
    val focused by interaction.collectIsFocusedAsState()
    Column(
        modifier
            .heightIn(min = 64.dp)
            .background(if (pressed) Colors.layerActive01 else Colors.layer01)
            .border(2.dp, if (focused) Colors.focus else Color.Transparent)
            .clickable(interaction, indication = null, role = Role.Button, onClick = onClick)
            .padding(Spacing.s05),
    ) {
        BasicText(title, style = titleStyle)
        if (detail != null) {
            BasicText(
                detail,
                style = Type.label01.copy(color = Colors.textSecondary),
                modifier = Modifier.padding(top = Spacing.s02),
            )
        }
    }
}

/**
 * Carbon's inline notification, warning kind (low contrast), with an
 * optional action at the right.
 */
@Composable
fun WarningNotification(
    title: String,
    text: String,
    modifier: Modifier = Modifier,
    action: String? = null,
    onAction: () -> Unit = {},
) {
    Row(
        modifier
            .fillMaxWidth()
            .height(IntrinsicSize.Min)
            .background(Colors.notificationBackgroundWarning)
            .border(1.dp, Colors.supportWarning.copy(alpha = 0.4f)),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(Modifier.width(3.dp).fillMaxHeight().background(Colors.supportWarning))
        Column(Modifier.weight(1f).padding(horizontal = Spacing.s05, vertical = Spacing.s04)) {
            BasicText(title, style = Type.headingCompact01.copy(color = Colors.textPrimary))
            BasicText(text, style = Type.bodyCompact01.copy(color = Colors.textPrimary))
        }
        if (action != null) {
            Button(
                action,
                onAction,
                kind = ButtonKind.Ghost,
            )
        }
    }
}
