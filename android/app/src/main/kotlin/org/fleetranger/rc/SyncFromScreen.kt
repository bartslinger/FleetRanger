package org.fleetranger.rc

import android.app.DatePickerDialog
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import java.time.Instant
import java.time.LocalDate
import java.time.ZoneId
import java.time.format.DateTimeFormatter
import java.util.Locale
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.fleetranger.rc.ui.Button
import org.fleetranger.rc.ui.Colors
import org.fleetranger.rc.ui.RadioButton
import org.fleetranger.rc.ui.Spacing
import org.fleetranger.rc.ui.Type

/** As in the web app, e.g. "12 Oct 2026". */
private val DATE = DateTimeFormatter.ofPattern("d MMM yyyy", Locale.ENGLISH)

/**
 * After pairing: which logs to upload, chosen on purpose rather than all of
 * them by default. From today is chosen to begin with, and is what the
 * controller uploads until someone continues. Each choice says how many of
 * the controller's logs it covers. `onChoose` gets the first day to upload,
 * or null for all logs.
 */
@Composable
fun SyncFromScreen(onChoose: (LocalDate?) -> Unit) {
    val context = LocalContext.current
    // The day of each log on the controller; null while looking.
    var days by remember { mutableStateOf<List<LocalDate>?>(null) }
    LaunchedEffect(Unit) {
        days = withContext(Dispatchers.IO) {
            FlightLogs(context).all().map {
                Instant.ofEpochMilli(it.file.lastModified()).atZone(ZoneId.systemDefault()).toLocalDate()
            }
        }
    }
    val today = remember { LocalDate.now() }
    // Null for all logs.
    var from by remember { mutableStateOf<LocalDate?>(today) }
    var aDate by remember { mutableStateOf(false) }

    fun pickDate() {
        val initial = from ?: today
        DatePickerDialog(
            context,
            { _, year, month, day ->
                from = LocalDate.of(year, month + 1, day)
                aDate = true
            },
            initial.year,
            initial.monthValue - 1,
            initial.dayOfMonth,
        ).show()
    }

    SyncFromContent(
        days = days,
        today = today,
        from = from,
        aDate = aDate,
        onToday = {
            from = today
            aDate = false
        },
        onAllLogs = {
            from = null
            aDate = false
        },
        onPickDate = ::pickDate,
        onContinue = { onChoose(from) },
    )
}

@Composable
private fun SyncFromContent(
    days: List<LocalDate>?,
    today: LocalDate,
    from: LocalDate?,
    aDate: Boolean,
    onToday: () -> Unit,
    onAllLogs: () -> Unit,
    onPickDate: () -> Unit,
    onContinue: () -> Unit,
) {
    // "12 logs", or "the oldest from …" for all of them; nothing while looking.
    fun count(from: LocalDate?): String? {
        if (days == null) return null
        val count = days.count { from == null || !it.isBefore(from) }
        val logs = "$count ${if (count == 1) "log" else "logs"}"
        val oldest = days.minOrNull()
        return if (from == null && oldest != null) "$logs, the oldest from ${DATE.format(oldest)}" else logs
    }

    Column(
        Modifier
            .fillMaxSize()
            .background(Colors.background)
            .verticalScroll(rememberScrollState())
            .padding(horizontal = Spacing.s07, vertical = Spacing.s06),
    ) {
        BasicText(
            "Which logs to upload",
            style = Type.heading04.copy(color = Colors.textPrimary),
            modifier = Modifier.semantics { heading() },
        )
        BasicText(
            if (days == null) {
                "Looking for flight logs on this controller…"
            } else {
                "FleetRanger uploads the flight logs on this controller from the day you choose on, " +
                    "except those it already has. You can change this later in Settings."
            },
            style = Type.bodyCompact02.copy(color = Colors.textPrimary),
            modifier = Modifier.widthIn(max = 600.dp).padding(top = Spacing.s05, bottom = Spacing.s03),
        )
        RadioButton("From today", selected = from == today && !aDate, onSelect = onToday, detail = count(today))
        RadioButton("All logs", selected = from == null, onSelect = onAllLogs, detail = count(null))
        Row(verticalAlignment = Alignment.CenterVertically) {
            RadioButton(
                "From a date",
                selected = aDate,
                onSelect = onPickDate,
                detail = if (aDate) count(from) else null,
            )
            if (aDate && from != null) {
                DateField(DATE.format(from), onPickDate, Modifier.padding(start = Spacing.s06))
            }
        }
        Button("Continue", onContinue, modifier = Modifier.padding(top = Spacing.s05))
    }
}

@Preview(widthDp = 768, heightDp = 432)
@Composable
private fun SyncFromScreenPreview() {
    val today = LocalDate.of(2026, 10, 20)
    SyncFromContent(
        days = List(40) { today.minusDays(it * 9L) },
        today = today,
        from = today,
        aDate = false,
        onToday = {},
        onAllLogs = {},
        onPickDate = {},
        onContinue = {},
    )
}
