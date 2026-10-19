package org.fleetranger.rc

import android.content.Context
import org.json.JSONArray
import org.json.JSONObject

/**
 * Someone who can be chosen as the pilot: an active user, with one of their
 * teams. A pilot in several teams is a choice per team, and their flights
 * belong to the team chosen.
 */
data class Pilot(val id: String, val name: String, val teamId: String, val teamName: String) {
    /** Unique in the list: the same person is there once per team. */
    val key get() = "$id:$teamId"
}

/** Who flies, as uploaded with a log: the pilot and the team they chose. */
data class Chosen(val pilotId: String, val teamId: String?)

/** The pilots this controller offers, as last downloaded. */
data class PilotList(
    /** The controller's team, whose pilots come first; null if it has none. */
    val teamId: String?,
    val pilots: List<Pilot>,
) {
    companion object {
        val EMPTY = PilotList(null, emptyList())

        /** From the server's answer to GET /api/controller/pilots. */
        fun fromJson(json: JSONObject): PilotList {
            val pilots = json.getJSONArray("pilots")
            return PilotList(
                teamId = json.optString("team_id").takeIf { !json.isNull("team_id") && it.isNotEmpty() },
                pilots = (0 until pilots.length()).map { i ->
                    val pilot = pilots.getJSONObject(i)
                    Pilot(
                        pilot.getString("id"),
                        pilot.getString("name"),
                        pilot.getString("team_id"),
                        pilot.getString("team_name"),
                    )
                },
            )
        }
    }

    fun toJson(): JSONObject = JSONObject()
        .put("team_id", teamId ?: JSONObject.NULL)
        .put(
            "pilots",
            JSONArray(
                pilots.map {
                    JSONObject().put("id", it.id).put("name", it.name)
                        .put("team_id", it.teamId).put("team_name", it.teamName)
                },
            ),
        )
}

/**
 * The pilot list, kept so the controller works offline, and who chose to fly
 * when: each choice is stored with its time, so a log can later be given the
 * pilot chosen most recently before it was written. Choices no waiting log
 * needs are forgotten, so they don't pile up.
 */
class Pilots(context: Context) {
    private val list = context.getSharedPreferences("pilot_list", Context.MODE_PRIVATE)
    private val choices = context.getSharedPreferences("pilot_choices", Context.MODE_PRIVATE)

    /** The list as last downloaded; empty before the first download. */
    var current: PilotList
        get() = list.getString("list", null)?.let { PilotList.fromJson(JSONObject(it)) }
            ?: PilotList.EMPTY
        set(value) = list.edit().putString("list", value.toJson().toString()).apply()

    /** Records a choice, as the pilot's id and team; `null` is "Not in the list". */
    fun choose(pilot: Pilot?, now: Long = System.currentTimeMillis()) {
        choices.edit().putString(now.toString(), pilot?.key ?: NOT_IN_THE_LIST).apply()
    }

    /**
     * Who flew a log last written at `time`: the one chosen most recently
     * before it, unless that choice is older than [EXPIRES_MS] (yesterday's
     * pilot isn't today's), or was "Not in the list". Null then.
     */
    fun at(time: Long): Chosen? {
        val (chosenAt, value) = choices.all
            .map { (at, value) -> at.toLong() to value as String }
            .filter { (at, _) -> at <= time }
            .maxByOrNull { (at, _) -> at } ?: return null
        if (value == NOT_IN_THE_LIST || time - chosenAt >= EXPIRES_MS) return null
        // Choices from before teams were part of them have the id alone.
        val parts = value.split(':')
        return Chosen(parts[0], parts.getOrNull(1)?.ifEmpty { null })
    }

    /**
     * Forgets the choices no log needs any more: those before `time`, the
     * oldest log still to upload, except the one that log falls under. With
     * nothing to upload, `time` is now and only the last choice stays.
     */
    fun forgetBefore(time: Long) {
        val older = choices.all.keys.map { it.toLong() }.filter { it <= time }.sorted()
        if (older.size < 2) return
        val editor = choices.edit()
        older.dropLast(1).forEach { editor.remove(it.toString()) }
        editor.apply()
    }

    /** The pilot chosen last, with the same team, if they can still be chosen. */
    fun last(): Pilot? {
        val value = choices.all.maxByOrNull { (time, _) -> time.toLong() }?.value
        return current.pilots.find { it.key == value }
    }

    private companion object {
        const val NOT_IN_THE_LIST = ""
        /** A choice counts for logs written within this time after it. */
        const val EXPIRES_MS = 6 * 3_600_000L
    }
}
