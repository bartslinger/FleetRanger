package org.fleetranger.rc.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
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
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

// Carbon's components, as far as the app needs them. Touch targets are at
// least 64dp, Carbon's extra large size: pilots may wear gloves.

/**
 * Carbon's button kinds. Danger is for a destructive action that is the
 * point of the screen (a confirmation); DangerTertiary for one among others.
 */
enum class ButtonKind { Primary, Secondary, Tertiary, Ghost, Danger, DangerTertiary }

/**
 * Carbon's button, extra large (64dp), with its text at the top left. While
 * `loading` is set, it shows that instead, with a spinner, and can't be tapped
 * again until the action is done.
 */
@Composable
fun Button(
    text: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    kind: ButtonKind = ButtonKind.Primary,
    loading: String? = null,
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
        ButtonKind.DangerTertiary -> if (pressed) Colors.buttonDangerActive else Color.Transparent
    }
    val foreground = when {
        kind == ButtonKind.Ghost -> Colors.interactive
        kind == ButtonKind.Tertiary && !pressed -> Colors.interactive
        kind == ButtonKind.DangerTertiary && !pressed -> Colors.buttonDanger
        else -> Colors.textOnColor
    }
    val border = when {
        focused -> Colors.focus
        kind == ButtonKind.Tertiary -> Colors.interactive
        kind == ButtonKind.DangerTertiary -> Colors.buttonDanger
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
            .clickable(interaction, indication = null, role = Role.Button, enabled = loading == null, onClick = onClick)
            .padding(
                start = Spacing.s05,
                end = if (ghost) Spacing.s05 else 64.dp,
                top = if (ghost) 0.dp else Spacing.s04,
            ),
        contentAlignment = if (ghost) Alignment.CenterStart else Alignment.TopStart,
    ) {
        if (loading != null) {
            InlineLoading(loading, color = foreground, style = Type.bodyCompact01)
        } else {
            BasicText(text, style = Type.bodyCompact01.copy(color = foreground))
        }
    }
}

/**
 * Carbon's clickable tile: one large target, a title and an optional line
 * under it. A border and an arrow at the bottom right show that it can be
 * tapped, which a white tile on light grey alone doesn't outdoors.
 */
@Composable
fun ClickableTile(
    title: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    detail: String? = null,
    titleStyle: TextStyle = Type.headingCompact02.copy(color = Colors.textPrimary),
    /**
     * What a tap does, e.g. "Open DJI Pilot 2": plain tile text, as Carbon
     * allows no link inside a clickable tile; the whole tile is the target.
     */
    action: String? = null,
    /** The arrow for somewhere in the app, Launch for another app. */
    icon: ImageVector = CarbonIcons.ArrowRight,
    /**
     * What the tile is, before the title, e.g. a pilot or a group: decorative,
     * in the text's colour and sized to it (Carbon's icon sizes: 20 beside
     * 16px text, 32 beside large type).
     */
    leading: ImageVector? = null,
    leadingSize: Dp = 20.dp,
) {
    val interaction = remember { MutableInteractionSource() }
    val pressed by interaction.collectIsPressedAsState()
    val focused by interaction.collectIsFocusedAsState()
    Box(
        modifier
            .heightIn(min = 64.dp)
            .background(if (pressed) Colors.layerActive01 else Colors.layer01)
            .border(
                if (focused) 2.dp else 1.dp,
                if (focused) Colors.focus else Colors.borderTile01,
            )
            .clickable(interaction, indication = null, role = Role.Button, onClick = onClick),
    ) {
        Column(Modifier.padding(Spacing.s05)) {
            if (leading != null) {
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(Spacing.s03),
                ) {
                    Icon(leading, Colors.iconPrimary, size = leadingSize)
                    BasicText(title, style = titleStyle)
                }
            } else {
                BasicText(title, style = titleStyle)
            }
            if (detail != null) {
                BasicText(
                    detail,
                    style = Type.label02.copy(color = Colors.textSecondary),
                    modifier = Modifier.padding(top = Spacing.s02),
                )
            }
        }
        if (action != null) {
            BasicText(
                action,
                style = Type.bodyCompact02.copy(color = Colors.textSecondary),
                modifier = Modifier.align(Alignment.BottomStart).padding(Spacing.s05),
            )
        }
        Icon(
            icon,
            Colors.interactive,
            Modifier.align(Alignment.BottomEnd).padding(Spacing.s04),
        )
    }
}

/** "Serial number 5Y…": the label in Plex Sans, the code itself in Plex Mono. */
@Composable
fun LabelledCode(label: String, code: String, modifier: Modifier = Modifier) {
    BasicText(
        buildAnnotatedString {
            append("$label ")
            withStyle(SpanStyle(fontFamily = PlexMono, letterSpacing = Type.code02.letterSpacing)) {
                append(code)
            }
        },
        style = Type.label02.copy(color = Colors.textSecondary),
        modifier = modifier,
    )
}

/** The statuses of Carbon's inline notification that the app uses. */
enum class NotificationKind { Error, Success, Warning }

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
) = InlineNotification(NotificationKind.Warning, title, text, modifier, action, onAction)

/**
 * Carbon's inline notification (low contrast), with an optional action at
 * the right. Not dismissible: it stays until what it says changes.
 */
@Composable
fun InlineNotification(
    kind: NotificationKind,
    title: String,
    text: String,
    modifier: Modifier = Modifier,
    action: String? = null,
    onAction: () -> Unit = {},
) {
    val (background, support) = when (kind) {
        NotificationKind.Error -> Colors.notificationBackgroundError to Colors.supportError
        NotificationKind.Success -> Colors.notificationBackgroundSuccess to Colors.supportSuccess
        NotificationKind.Warning -> Colors.notificationBackgroundWarning to Colors.supportWarning
    }
    Row(
        modifier
            .fillMaxWidth()
            .height(IntrinsicSize.Min)
            .background(background)
            .border(1.dp, support.copy(alpha = 0.4f))
            // Read out when it appears, like role="alert" or "status" on the web.
            .semantics {
                liveRegion = if (kind == NotificationKind.Error) LiveRegionMode.Assertive else LiveRegionMode.Polite
            },
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(Modifier.width(3.dp).fillMaxHeight().background(support))
        // Status by icon and colour, not colour alone; aligned with the title.
        val iconModifier = Modifier.align(Alignment.Top).padding(start = Spacing.s05, top = Spacing.s04 + 1.dp)
        when (kind) {
            NotificationKind.Error -> ErrorIcon(iconModifier)
            NotificationKind.Success -> SuccessIcon(iconModifier)
            NotificationKind.Warning -> WarningIcon(iconModifier)
        }
        Column(Modifier.weight(1f).padding(horizontal = Spacing.s05, vertical = Spacing.s04)) {
            BasicText(title, style = Type.headingCompact02.copy(color = Colors.textPrimary))
            BasicText(text, style = Type.bodyCompact02.copy(color = Colors.textPrimary))
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
