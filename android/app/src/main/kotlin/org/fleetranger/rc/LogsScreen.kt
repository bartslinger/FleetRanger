package org.fleetranger.rc

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import java.time.Instant
import java.time.ZoneId
import java.time.format.DateTimeFormatter
import java.util.Locale
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.fleetranger.rc.ui.Colors
import org.fleetranger.rc.ui.Spacing
import org.fleetranger.rc.ui.TopBar
import org.fleetranger.rc.ui.Type
import org.fleetranger.rc.ui.WarningIcon

private val WRITTEN = DateTimeFormatter.ofPattern("d MMM yyyy, HH:mm", Locale.ENGLISH)

/**
 * Every flight log on the controller and where it stands, newest first. Read
 * again whenever the upload status changes.
 */
@Composable
fun LogsScreen(syncStatus: SyncStatus?, onBack: () -> Unit) {
    BackHandler(onBack = onBack)
    val context = LocalContext.current
    var logs by remember { mutableStateOf<List<FlightLogs.Log>?>(null) }
    LaunchedEffect(syncStatus) {
        logs = withContext(Dispatchers.IO) { FlightLogs(context).all() }
    }

    Column(Modifier.fillMaxSize().background(Colors.background)) {
        TopBar("Flight logs", onBack)
        LazyColumn(
            Modifier
                .weight(1f)
                .fillMaxWidth()
                .padding(start = Spacing.s05, end = Spacing.s05, bottom = Spacing.s05)
                .background(Colors.layer01),
        ) {
            val shown = logs ?: return@LazyColumn
            if (shown.isEmpty()) {
                item { Empty("No flight logs on this controller yet") }
            }
            items(shown, key = { it.file.path }) { LogRow(it) }
        }
    }
}

/** A log: its file name, when it was written and its size, then its state. */
@Composable
private fun LogRow(log: FlightLogs.Log) {
    Row(
        Modifier
            .fillMaxWidth()
            .heightIn(min = 64.dp)
            .drawBehind {
                drawLine(
                    Colors.borderSubtle01,
                    Offset(0f, size.height - 0.5f),
                    Offset(size.width, size.height - 0.5f),
                    1.dp.toPx(),
                )
            }
            .padding(horizontal = Spacing.s05, vertical = Spacing.s04),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Column(Modifier.weight(1f)) {
            BasicText(log.file.name, style = Type.bodyCompact02.copy(color = Colors.textPrimary))
            BasicText(
                WRITTEN.format(Instant.ofEpochMilli(log.file.lastModified()).atZone(ZoneId.systemDefault())) +
                    " · " + size(log.file.length()),
                style = Type.label02.copy(color = Colors.textSecondary),
            )
        }
        // Waiting logs get the warning icon, as on the other screens.
        Row(
            Modifier.padding(start = Spacing.s05),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            if (log.state == FlightLogs.State.Waiting) WarningIcon(Modifier.padding(end = Spacing.s03))
            BasicText(
                when (log.state) {
                    FlightLogs.State.Uploaded -> "Uploaded"
                    FlightLogs.State.Waiting -> "Waiting"
                    FlightLogs.State.BeingWritten -> "Being written"
                    FlightLogs.State.BeforeSyncDate -> "Before the sync date"
                },
                style = Type.bodyCompact02.copy(
                    color = if (log.state == FlightLogs.State.Waiting) Colors.textPrimary else Colors.textSecondary,
                ),
            )
        }
    }
}

@Composable
private fun Empty(text: String) {
    BasicText(
        text,
        style = Type.bodyCompact02.copy(color = Colors.textSecondary),
        modifier = Modifier.padding(Spacing.s05),
    )
}

/** As people read file sizes: "640 kB", "2.1 MB". */
private fun size(bytes: Long): String =
    if (bytes < 1_000_000) "${(bytes + 999) / 1000} kB" else String.format(Locale.ENGLISH, "%.1f MB", bytes / 1e6)
