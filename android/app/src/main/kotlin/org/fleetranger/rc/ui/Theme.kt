package org.fleetranger.rc.ui

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import androidx.compose.ui.unit.sp
import org.fleetranger.rc.R

// Carbon's tokens, with the web app's values (web/src/lib/ui/tokens.css). The
// app is light only, Gray 10, like DJI Pilot 2 beside it. Only the tokens the
// app uses are here; add more from tokens.css as components need them.

/** Carbon's Gray 10 theme. */
object Colors {
    val background = Color(0xFFF4F4F4)
    val layer01 = Color(0xFFFFFFFF)
    val layerHover01 = Color(0xFFE8E8E8)
    val layerActive01 = Color(0xFFC6C6C6)
    val borderSubtle01 = Color(0xFFE0E0E0)
    val textPrimary = Color(0xFF161616)
    val textSecondary = Color(0xFF525252)
    val textOnColor = Color(0xFFFFFFFF)
    val interactive = Color(0xFF0F62FE)
    val focus = Color(0xFF0F62FE)
    val buttonPrimary = Color(0xFF0F62FE)
    val buttonPrimaryActive = Color(0xFF002D9C)
    val buttonSecondary = Color(0xFF393939)
    val buttonSecondaryActive = Color(0xFF6F6F6F)
    val buttonTertiaryActive = Color(0xFF002D9C)
    val buttonDanger = Color(0xFFDA1E28)
    val buttonDangerActive = Color(0xFF750E13)
    val borderStrong01 = Color(0xFF8D8D8D)
    val iconPrimary = Color(0xFF161616)
    val overlay = Color(0x80161616)
    val supportWarning = Color(0xFFF1C21B)
    val notificationBackgroundWarning = Color(0xFFFCF4D6)
}

val Plex = FontFamily(
    Font(R.font.ibm_plex_sans_regular, FontWeight.Normal),
    Font(R.font.ibm_plex_sans_semibold, FontWeight.SemiBold),
)

/** Carbon's type tokens (productive set). */
object Type {
    val heading03 = TextStyle(fontFamily = Plex, fontSize = 20.sp, lineHeight = 28.sp)
    val heading04 = TextStyle(fontFamily = Plex, fontSize = 28.sp, lineHeight = 36.sp)
    val headingCompact01 = TextStyle(
        fontFamily = Plex,
        fontWeight = FontWeight.SemiBold,
        fontSize = 14.sp,
        lineHeight = 18.sp,
        letterSpacing = 0.16.sp,
    )
    val headingCompact02 = TextStyle(
        fontFamily = Plex,
        fontWeight = FontWeight.SemiBold,
        fontSize = 16.sp,
        lineHeight = 22.sp,
    )
    val bodyCompact01 = TextStyle(
        fontFamily = Plex,
        fontSize = 14.sp,
        lineHeight = 18.sp,
        letterSpacing = 0.16.sp,
    )
    val label01 = TextStyle(
        fontFamily = Plex,
        fontSize = 12.sp,
        lineHeight = 16.sp,
        letterSpacing = 0.032.em,
    )
}

/** Carbon's spacing scale. */
object Spacing {
    val s01 = 2.dp
    val s02 = 4.dp
    val s03 = 8.dp
    val s04 = 12.dp
    val s05 = 16.dp
    val s06 = 24.dp
    val s07 = 32.dp
}
