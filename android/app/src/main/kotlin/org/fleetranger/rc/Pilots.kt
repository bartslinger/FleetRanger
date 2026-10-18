package org.fleetranger.rc

import android.content.Context

/** Someone who can be chosen as the pilot. */
data class Pilot(val id: String, val name: String)

/**
 * Who chose to fly when. Each choice is stored with its time, so a log can
 * later be given the pilot chosen most recently before it was written.
 */
class Pilots(context: Context) {
    private val prefs = context.getSharedPreferences("pilot_choices", Context.MODE_PRIVATE)

    /**
     * The users who can be chosen. Test names until the controller is paired
     * with an install and downloads the real list.
     */
    val all: List<Pilot> = listOf(
        Pilot("test-a", "Test pilot A"),
        Pilot("test-b", "Test pilot B"),
        Pilot("test-c", "Test pilot C"),
        Pilot("test-d", "Test pilot D"),
    )

    /** Records a choice; `null` is "Not in the list". */
    fun choose(pilot: Pilot?, now: Long = System.currentTimeMillis()) {
        prefs.edit().putString(now.toString(), pilot?.id ?: NOT_IN_THE_LIST).apply()
    }

    /** The pilot chosen last, if they can still be chosen. */
    fun last(): Pilot? {
        val id = prefs.all.maxByOrNull { (time, _) -> time.toLong() }?.value
        return all.find { it.id == id }
    }

    private companion object {
        const val NOT_IN_THE_LIST = ""
    }
}
