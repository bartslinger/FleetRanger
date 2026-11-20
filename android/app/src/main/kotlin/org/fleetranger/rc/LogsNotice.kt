package org.fleetranger.rc

import androidx.compose.runtime.Composable
import org.fleetranger.rc.ui.InlineNotification
import org.fleetranger.rc.ui.NotificationKind

/**
 * What the main screen says when the app can't get at DJI Pilot 2's flight
 * logs, so a pilot doesn't expect uploads that won't come (user's decisions,
 * 2026-11-20). Kinds and texts after Carbon's notification guidelines.
 */
sealed interface LogsNotice {
    /** No FlightRecord folder in the usual places: perhaps never flown, perhaps moved. */
    data class NotFound(val searching: Boolean) : LogsNotice

    /** A search found a folder elsewhere, which the app uses from now on. */
    data object Found : LogsNotice

    /** A search found nothing. */
    data class NothingFound(val support: String?, val searching: Boolean = false) : LogsNotice

    /** A folder the app may not read: logs won't come until someone fixes it. */
    data class Unreadable(val support: String?) : LogsNotice

    companion object {
        /** The notice for the folders' state; null when the logs are within reach. */
        fun of(check: FlightLogs.Check, support: String?): LogsNotice? = when (check.access) {
            FlightLogs.Access.Found -> null
            FlightLogs.Access.Unreadable -> Unreadable(support)
            FlightLogs.Access.NotFound ->
                if (check.searchedInVain) NothingFound(support) else NotFound(searching = false)
        }
    }
}

/** Who to contact: the install's support address, or whoever set it up. */
private fun contact(support: String?) =
    if (support != null) "Contact $support." else "Ask whoever set up FleetRanger."

@Composable
fun LogsNotification(notice: LogsNotice, onSearch: () -> Unit) {
    when (notice) {
        is LogsNotice.NotFound -> InlineNotification(
            NotificationKind.Info,
            title = "No flight logs found",
            text = "DJI Pilot 2 may keep them in another folder.",
            action = "Search controller",
            onAction = onSearch,
            busy = if (notice.searching) "Searching…" else null,
        )
        LogsNotice.Found -> InlineNotification(
            NotificationKind.Success,
            title = "Flight logs found",
            text = "FleetRanger uses this folder from now on.",
        )
        is LogsNotice.NothingFound -> InlineNotification(
            NotificationKind.Info,
            title = "No flight logs on this controller",
            text = "Flown already? ${contact(notice.support)}",
            // DJI Pilot 2 may write its logs somewhere new after this search.
            action = "Search again",
            onAction = onSearch,
            busy = if (notice.searching) "Searching…" else null,
        )
        is LogsNotice.Unreadable -> InlineNotification(
            NotificationKind.Error,
            title = "Can’t read the flight logs",
            text = contact(notice.support),
        )
    }
}
