package org.fleetranger.rc

import android.content.Context
import org.json.JSONArray
import org.json.JSONObject

/** Someone who can be chosen as the pilot: an active user in a team. */
data class Pilot(val id: String, val name: String, val teamId: String, val teamName: String)

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
 * pilot chosen most recently before it was written.
 */
class Pilots(context: Context) {
    private val list = context.getSharedPreferences("pilot_list", Context.MODE_PRIVATE)
    private val choices = context.getSharedPreferences("pilot_choices", Context.MODE_PRIVATE)

    /** The list as last downloaded; empty before the first download. */
    var current: PilotList
        get() = list.getString("list", null)?.let { PilotList.fromJson(JSONObject(it)) }
            ?: PilotList.EMPTY
        set(value) = list.edit().putString("list", value.toJson().toString()).apply()

    /** Records a choice; `null` is "Not in the list". */
    fun choose(pilot: Pilot?, now: Long = System.currentTimeMillis()) {
        choices.edit().putString(now.toString(), pilot?.id ?: NOT_IN_THE_LIST).apply()
    }

    /** The pilot chosen last, if they can still be chosen. */
    fun last(): Pilot? {
        val id = choices.all.maxByOrNull { (time, _) -> time.toLong() }?.value
        return current.pilots.find { it.id == id }
    }

    private companion object {
        const val NOT_IN_THE_LIST = ""
    }
}
