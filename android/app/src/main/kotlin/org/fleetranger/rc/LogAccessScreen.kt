package org.fleetranger.rc

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import org.fleetranger.rc.ui.Button
import org.fleetranger.rc.ui.Colors
import org.fleetranger.rc.ui.Spacing
import org.fleetranger.rc.ui.Type

/**
 * Before Android asks for access to files: what the dialog will ask and why,
 * so whoever sets up the controller knows to allow it. After a refusal,
 * Android doesn't ask again, so the button opens the app's Android settings.
 */
@Composable
fun LogAccessScreen(refused: Boolean, onContinue: () -> Unit, onOpenAppSettings: () -> Unit) {
    val text = Type.bodyCompact01.copy(color = Colors.textPrimary, fontSize = 16.sp, lineHeight = 22.sp)
    Column(
        Modifier
            .fillMaxSize()
            .background(Colors.background)
            .padding(horizontal = Spacing.s07, vertical = Spacing.s06),
        verticalArrangement = Arrangement.spacedBy(Spacing.s05),
    ) {
        BasicText(
            "Allow access to the flight logs",
            style = Type.heading04.copy(color = Colors.textPrimary),
            modifier = Modifier.semantics { heading() },
        )
        Column(Modifier.widthIn(max = 600.dp), verticalArrangement = Arrangement.spacedBy(Spacing.s05)) {
            BasicText(
                "FleetRanger uploads the flight logs that DJI Pilot 2 keeps on this controller, " +
                    "so they appear in the logbook.",
                style = text,
            )
            if (!refused) {
                BasicText(
                    buildAnnotatedString {
                        append("Android will now ask whether FleetRanger may access photos, media and files. Choose ")
                        withStyle(SpanStyle(fontWeight = FontWeight.SemiBold)) { append("Allow") }
                        append(". FleetRanger only reads the flight logs.")
                    },
                    style = text,
                )
            } else {
                BasicText(
                    buildAnnotatedString {
                        append("Access was refused, so no logs are uploaded. In Android’s settings for ")
                        append("FleetRanger, open ")
                        withStyle(SpanStyle(fontWeight = FontWeight.SemiBold)) { append("Permissions") }
                        append(", then ")
                        withStyle(SpanStyle(fontWeight = FontWeight.SemiBold)) { append("Storage") }
                        append(", and choose ")
                        withStyle(SpanStyle(fontWeight = FontWeight.SemiBold)) { append("Allow") }
                        append(".")
                    },
                    style = text,
                )
            }
        }
        if (!refused) {
            Button("Continue", onContinue, modifier = Modifier.padding(top = Spacing.s05))
        } else {
            Button("Open Android settings", onOpenAppSettings, modifier = Modifier.padding(top = Spacing.s05))
        }
    }
}

@Preview(widthDp = 768, heightDp = 432)
@Composable
private fun LogAccessScreenPreview() {
    LogAccessScreen(refused = false, onContinue = {}, onOpenAppSettings = {})
}
