package org.fleetranger.rc.ui

import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsFocusedAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog

/**
 * Carbon's text input: a label, a 64dp field (for gloves) on the page's
 * background in field-01, and either helper text or, when `invalid` is set,
 * the error with a red outline and icon.
 */
@Composable
fun TextInput(
    label: String,
    value: String,
    onValueChange: (String) -> Unit,
    modifier: Modifier = Modifier,
    placeholder: String = "",
    helper: String? = null,
    invalid: String? = null,
    enabled: Boolean = true,
    keyboardOptions: KeyboardOptions = KeyboardOptions.Default,
    onDone: () -> Unit = {},
) {
    val interaction = remember { MutableInteractionSource() }
    val focused by interaction.collectIsFocusedAsState()
    Column(modifier, verticalArrangement = Arrangement.spacedBy(Spacing.s03)) {
        BasicText(label, style = Type.label02.copy(color = Colors.textSecondary))
        BasicTextField(
            value,
            onValueChange,
            enabled = enabled,
            singleLine = true,
            textStyle = Type.bodyCompact02.copy(color = Colors.textPrimary),
            cursorBrush = SolidColor(Colors.focus),
            keyboardOptions = keyboardOptions,
            keyboardActions = KeyboardActions(onDone = { onDone() }, onGo = { onDone() }),
            interactionSource = interaction,
            decorationBox = { field ->
                Row(
                    Modifier
                        .fillMaxWidth()
                        .height(64.dp)
                        .background(Colors.field01)
                        .then(
                            when {
                                invalid != null -> Modifier.border(2.dp, Colors.supportError)
                                focused -> Modifier.border(2.dp, Colors.focus)
                                else -> Modifier.drawBehind {
                                    drawLine(
                                        Colors.borderStrong01,
                                        Offset(0f, size.height - 0.5f),
                                        Offset(size.width, size.height - 0.5f),
                                        1.dp.toPx(),
                                    )
                                }
                            },
                        )
                        .padding(horizontal = Spacing.s05),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Box(Modifier.weight(1f)) {
                        if (value.isEmpty()) {
                            BasicText(placeholder, style = Type.bodyCompact02.copy(color = Colors.textPlaceholder))
                        }
                        field()
                    }
                    if (invalid != null) ErrorIcon(Modifier.padding(start = Spacing.s05))
                }
            },
        )
        when {
            invalid != null -> BasicText(invalid, style = Type.label02.copy(color = Colors.textError))
            helper != null -> BasicText(helper, style = Type.label02.copy(color = Colors.textHelper))
        }
    }
}

/**
 * Carbon's radio button, with its label: the whole row is the target, 64dp
 * high for gloves. `detail` is an optional second line.
 */
@Composable
fun RadioButton(
    label: String,
    selected: Boolean,
    onSelect: () -> Unit,
    modifier: Modifier = Modifier,
    detail: String? = null,
) {
    Row(
        modifier
            .heightIn(min = 64.dp)
            .selectable(
                selected = selected,
                interactionSource = remember { MutableInteractionSource() },
                indication = null,
                role = Role.RadioButton,
                onClick = onSelect,
            )
            // Centres a one-line label in the 64dp row; with a detail, the
            // two lines fill it.
            .padding(vertical = if (detail == null) 21.dp else Spacing.s04),
        // With two lines, Carbon aligns the circle with the first one.
        verticalAlignment = Alignment.Top,
        horizontalArrangement = Arrangement.spacedBy(Spacing.s03),
    ) {
        // 20dp, a 1dp ring, an 8dp dot when selected; centred on the 22dp first line.
        Box(
            Modifier
                .padding(top = 1.dp)
                .size(20.dp)
                .border(1.dp, Colors.iconPrimary, CircleShape),
            contentAlignment = Alignment.Center,
        ) {
            if (selected) Box(Modifier.size(8.dp).background(Colors.iconPrimary, CircleShape))
        }
        Column {
            BasicText(label, style = Type.bodyCompact02.copy(color = Colors.textPrimary))
            if (detail != null) {
                BasicText(detail, style = Type.label02.copy(color = Colors.textSecondary))
            }
        }
    }
}

/**
 * Carbon's inline loading: a small spinner before the text, for something the
 * app is doing right now, such as pairing.
 */
@Composable
fun InlineLoading(text: String, modifier: Modifier = Modifier) {
    val turning = rememberInfiniteTransition(label = "loading")
    val angle by turning.animateFloat(
        initialValue = 0f,
        targetValue = 360f,
        animationSpec = infiniteRepeatable(tween(690, easing = LinearEasing)),
        label = "angle",
    )
    Row(
        modifier,
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(Spacing.s03),
    ) {
        // Carbon's small loading: a 16dp ring, a quarter of it in the
        // interactive colour, turning.
        Canvas(Modifier.size(16.dp)) {
            val stroke = Stroke(width = 2.dp.toPx())
            drawCircle(Colors.borderSubtle01, style = stroke, radius = size.minDimension / 2 - stroke.width / 2)
            drawArc(
                Colors.interactive,
                startAngle = angle,
                sweepAngle = 90f,
                useCenter = false,
                topLeft = androidx.compose.ui.geometry.Offset(stroke.width / 2, stroke.width / 2),
                size = androidx.compose.ui.geometry.Size(size.width - stroke.width, size.height - stroke.width),
                style = stroke,
            )
        }
        BasicText(text, style = Type.bodyCompact02.copy(color = Colors.textSecondary))
    }
}

/**
 * Carbon's danger modal: a title, what will happen, and Cancel next to the
 * dangerous action, the two buttons sharing the footer.
 */
@Composable
fun DangerDialog(
    title: String,
    text: String,
    action: String,
    onConfirm: () -> Unit,
    onDismiss: () -> Unit,
) {
    Dialog(onDismissRequest = onDismiss) {
        Column(Modifier.width(480.dp).background(Colors.layer01)) {
            Column(
                Modifier.padding(Spacing.s05),
                verticalArrangement = Arrangement.spacedBy(Spacing.s05),
            ) {
                BasicText(title, style = Type.heading03.copy(color = Colors.textPrimary))
                BasicText(text, style = Type.bodyCompact02.copy(color = Colors.textPrimary))
            }
            Row(Modifier.fillMaxWidth().padding(top = Spacing.s07)) {
                Button(
                    "Cancel",
                    onDismiss,
                    kind = ButtonKind.Secondary,
                    modifier = Modifier.weight(1f),
                )
                Button(action, onConfirm, kind = ButtonKind.Danger, modifier = Modifier.weight(1f))
            }
        }
    }
}
