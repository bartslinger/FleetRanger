package org.fleetranger.rc.ui

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.PathFillType
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.graphics.vector.PathParser
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

// Carbon's icons (@carbon/icons, Apache-2.0, copyright IBM Corp.), as the web
// app uses them: the 20px versions' path data, copied as is.

/** A Carbon icon drawn in one colour. */
private fun icon(name: String, viewBox: Float, vararg paths: String): ImageVector =
    ImageVector.Builder(name, 20.dp, 20.dp, viewBox, viewBox).apply {
        paths.forEach { path ->
            addPath(
                pathData = PathParser().parsePathString(path).toNodes(),
                fill = SolidColor(Color.Black),
                pathFillType = PathFillType.EvenOdd,
            )
        }
    }.build()

object CarbonIcons {
    val ArrowRight = icon("arrow--right", 20f, "M11.8 2.8 10.8 3.8 16.2 9.3 1 9.3 1 10.7 16.2 10.7 10.8 16.2 11.8 17.2 19 10z")
    val Calendar = icon(
        "calendar",
        32f,
        "M26,4h-4V2h-2v2h-8V2h-2v2H6C4.9,4,4,4.9,4,6v20c0,1.1,0.9,2,2,2h20c1.1,0,2-0.9,2-2V6C28,4.9,27.1,4,26,4z " +
            "M26,26H6V12h20V26z M26,10H6V6h4v2h2V6h8v2h2V6h4V10z",
    )

    /** The circle, with the exclamation mark cut out; drawn over [WarningMark]. */
    val Warning = icon(
        "warning--filled",
        20f,
        "M10,1c-5,0-9,4-9,9s4,9,9,9s9-4,9-9S15,1,10,1z M9.2,5h1.5v7H9.2V5z " +
            "M10,16c-0.6,0-1-0.4-1-1s0.4-1,1-1s1,0.4,1,1S10.6,16,10,16z",
    )
    val WarningMark = icon(
        "warning--filled-inner",
        20f,
        "M9.2,5h1.5v7H9.2V5z M10,16c-0.6,0-1-0.4-1-1s0.4-1,1-1s1,0.4,1,1S10.6,16,10,16z",
    )
}

@Composable
fun Icon(icon: ImageVector, color: Color, modifier: Modifier = Modifier, size: Dp = 20.dp) {
    Image(icon, contentDescription = null, modifier.size(size), colorFilter = ColorFilter.tint(color))
}

/** Carbon's warning status icon: a yellow circle with a black exclamation mark. */
@Composable
fun WarningIcon(modifier: Modifier = Modifier) {
    androidx.compose.foundation.layout.Box(modifier.size(20.dp)) {
        Icon(CarbonIcons.WarningMark, Colors.textPrimary)
        Icon(CarbonIcons.Warning, Colors.supportWarning)
    }
}
