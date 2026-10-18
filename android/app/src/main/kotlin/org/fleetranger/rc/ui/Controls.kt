package org.fleetranger.rc.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog

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
            ),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(Spacing.s03),
    ) {
        // 20dp, a 1dp ring; selected: a 2dp ring around a dot.
        Box(
            Modifier
                .size(20.dp)
                .border(if (selected) 2.dp else 1.dp, Colors.iconPrimary, CircleShape),
            contentAlignment = Alignment.Center,
        ) {
            if (selected) Box(Modifier.size(10.dp).background(Colors.iconPrimary, CircleShape))
        }
        Column {
            BasicText(label, style = Type.bodyCompact01.copy(color = Colors.textPrimary))
            if (detail != null) {
                BasicText(detail, style = Type.label01.copy(color = Colors.textSecondary))
            }
        }
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
                BasicText(text, style = Type.bodyCompact01.copy(color = Colors.textPrimary))
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
