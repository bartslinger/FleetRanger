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
import androidx.compose.foundation.layout.IntrinsicSize
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
import androidx.compose.foundation.text.selection.LocalTextSelectionColors
import androidx.compose.foundation.text.selection.TextSelectionColors
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.error
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.text.input.KeyboardCapitalization
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
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
    /** Locked while the app works with the value, but still read out. */
    readOnly: Boolean = false,
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
            readOnly = readOnly,
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
 * A short code typed one character per box, the way it's shown elsewhere,
 * with a dash between the halves (KDW4-9TXR). Carbon has no such component:
 * this is one text field, so typing, pasting, backspace and TalkBack work as
 * in any field, drawn as boxes in Carbon's field style. The box the next
 * character goes into shows focus; when invalid, all boxes do in red. Label,
 * helper and error are the group's, as with [TextInput]. The value holds
 * only the code's characters: others, such as a pasted dash, are dropped.
 * The keyboard's Done only closes the keyboard, as Android intends; the
 * screen's button submits.
 */
@Composable
fun CodeInput(
    label: String,
    value: String,
    onValueChange: (String) -> Unit,
    length: Int,
    /** The characters a code can have, in upper case. */
    alphabet: String,
    modifier: Modifier = Modifier,
    helper: String? = null,
    invalid: String? = null,
    readOnly: Boolean = false,
) {
    val interaction = remember { MutableInteractionSource() }
    val focused by interaction.collectIsFocusedAsState()
    // The text with its selection, so a selection (e.g. "Select all") can show
    // on the boxes; a value changed from outside starts with the cursor at the end.
    var edited by remember { mutableStateOf(TextFieldValue(value, TextRange(value.length))) }
    val field = if (edited.text == value) edited else TextFieldValue(value, TextRange(value.length))
    val selected = if (field.selection.collapsed) IntRange.EMPTY else field.selection.min until field.selection.max
    // As wide as the boxes, so helper and error text wrap there.
    Column(modifier.width(IntrinsicSize.Min), verticalArrangement = Arrangement.spacedBy(Spacing.s03)) {
        BasicText(label, style = Type.label02.copy(color = Colors.textSecondary))
        // The hidden field's own selection handles and highlight would show
        // where it lies, over the first box; the boxes show the selection.
        CompositionLocalProvider(
            LocalTextSelectionColors provides TextSelectionColors(Color.Transparent, Color.Transparent),
        ) {
            BasicTextField(
                field,
                { typed ->
                    val text = typed.text.uppercase().filter { it in alphabet }.take(length)
                    // Typed as it should be, it keeps its selection; changed, the cursor goes to the end.
                    edited = if (text == typed.text) typed else TextFieldValue(text, TextRange(text.length))
                    if (text != value) onValueChange(text)
                },
                modifier = Modifier.semantics {
                    contentDescription = label
                    invalid?.let { error(it) }
                },
                readOnly = readOnly,
                singleLine = true,
                // The boxes show the characters, and where the next one goes.
                textStyle = TextStyle(color = Color.Transparent, fontSize = 1.sp),
                cursorBrush = SolidColor(Color.Transparent),
                keyboardOptions = KeyboardOptions(
                    capitalization = KeyboardCapitalization.Characters,
                    autoCorrectEnabled = false,
                    keyboardType = KeyboardType.Ascii,
                    imeAction = ImeAction.Done,
                ),
                interactionSource = interaction,
                decorationBox = { input ->
                    Box {
                        // The real field, invisible, under the boxes: it takes the
                        // keyboard's input. First, so it keeps its place when the
                        // error icon comes and goes: moved, it would be made anew.
                        // At the bottom, with tiny text: the cursor it scrolls into
                        // view lies below the helper or error, so all of the field
                        // shows above the keyboard.
                        Box(Modifier.align(Alignment.BottomStart).size(1.dp)) { input() }
                        Column(verticalArrangement = Arrangement.spacedBy(Spacing.s03)) {
                            Row(verticalAlignment = Alignment.CenterVertically) {
                                // Drawn from the value; the field itself is read out.
                                Row(
                                    Modifier.clearAndSetSemantics {},
                                    horizontalArrangement = Arrangement.spacedBy(Spacing.s02),
                                    verticalAlignment = Alignment.CenterVertically,
                                ) {
                                    for (i in 0 until length) {
                                        if (i == length / 2) {
                                            BasicText(
                                                "-",
                                                style = Type.heading04.copy(color = Colors.textSecondary, fontFamily = PlexMono),
                                            )
                                        }
                                        CodeBox(
                                            char = value.getOrNull(i),
                                            current = focused && !readOnly && selected.isEmpty() &&
                                                i == value.length.coerceAtMost(length - 1),
                                            selected = i in selected,
                                            invalid = invalid != null,
                                        )
                                    }
                                }
                                if (invalid != null) ErrorIcon(Modifier.padding(start = Spacing.s05))
                            }
                            when {
                                invalid != null -> BasicText(invalid, style = Type.label02.copy(color = Colors.textError))
                                helper != null -> BasicText(helper, style = Type.label02.copy(color = Colors.textHelper))
                            }
                        }
                    }
                },
            )
        }
    }
}

/** One character's box: Carbon's field, 64dp high for gloves. */
@Composable
private fun CodeBox(char: Char?, current: Boolean, selected: Boolean, invalid: Boolean) {
    Box(
        Modifier
            .size(width = 52.dp, height = 64.dp)
            // Selected text in Carbon's highlight colour.
            .background(if (selected) Colors.highlight else Colors.field01)
            .then(
                when {
                    invalid -> Modifier.border(2.dp, Colors.supportError)
                    current -> Modifier.border(2.dp, Colors.focus)
                    else -> Modifier.drawBehind {
                        drawLine(
                            Colors.borderStrong01,
                            Offset(0f, size.height - 0.5f),
                            Offset(size.width, size.height - 0.5f),
                            1.dp.toPx(),
                        )
                    }
                },
            ),
        contentAlignment = Alignment.Center,
    ) {
        BasicText(
            char?.toString().orEmpty(),
            style = Type.heading04.copy(color = Colors.textPrimary, fontFamily = PlexMono, textAlign = TextAlign.Center),
        )
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
fun InlineLoading(
    text: String,
    modifier: Modifier = Modifier,
    /** The text and turning arc; on a coloured button, its text colour. */
    color: Color? = null,
    style: TextStyle = Type.bodyCompact02,
) {
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
            drawCircle(
                color?.copy(alpha = 0.3f) ?: Colors.borderSubtle01,
                style = stroke,
                radius = size.minDimension / 2 - stroke.width / 2,
            )
            drawArc(
                color ?: Colors.interactive,
                startAngle = angle,
                sweepAngle = 90f,
                useCenter = false,
                topLeft = androidx.compose.ui.geometry.Offset(stroke.width / 2, stroke.width / 2),
                size = androidx.compose.ui.geometry.Size(size.width - stroke.width, size.height - stroke.width),
                style = stroke,
            )
        }
        BasicText(text, style = style.copy(color = color ?: Colors.textSecondary))
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
