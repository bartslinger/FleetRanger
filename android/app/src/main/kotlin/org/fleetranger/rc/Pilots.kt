package org.fleetranger.rc

import android.content.Context
import org.json.JSONArray
import org.json.JSONObject

/** A group of the organisation, which flights belong to. */
data class Group(val id: String, val name: String)

/**
 * Someone who can be chosen as the pilot: an active user who flies, with the
 * groups they're a member of and fly for, and the other groups they may fly
 * for (see "Flights and pilots" in docs/architecture.md).
 */
data class Pilot(val id: String, val name: String, val groups: List<String>, val otherGroups: List<String>)

/**
 * Who flies, as chosen on the controller: the pilot, and the group the
 * flights are for; null when the organisation has no groups.
 */
data class Choice(val pilot: Pilot, val groupId: String?) {
    /** As stored with the time it was chosen. */
    val key get() = "${pilot.id}:${groupId.orEmpty()}"
}

/** Who flies, as uploaded with a log: the pilot and the group they chose. */
data class Chosen(val pilotId: String, val groupId: String?)

/** The pilots this controller offers, as last downloaded. */
data class PilotList(
    /** The controller's group, whose pilots come first; null if it has none. */
    val controllerGroup: String?,
    /** Every group, by name; empty when the organisation has none. */
    val groups: List<Group>,
    val pilots: List<Pilot>,
) {
    fun group(id: String?): Group? = groups.find { it.id == id }

    /** Every group this pilot may fly for: their own first. */
    private fun groupsFor(pilot: Pilot): List<String> = pilot.groups + pilot.otherGroups

    /**
     * Whether the pilot chooses a group, on a screen of its own after the
     * pilot: when they may fly for more than one (user's decision,
     * 2026-10-23). Otherwise the one is picked by itself.
     */
    fun choosesGroup(pilot: Pilot): Boolean = groupsFor(pilot).size > 1

    /**
     * The choice for this pilot and group, if it can still be made: a group
     * they may fly for; with no groups in the organisation, none.
     */
    fun choice(pilot: Pilot, groupId: String?): Choice? {
        if (groups.isEmpty()) return Choice(pilot, null)
        if (groupId != null && groupId in groupsFor(pilot)) return Choice(pilot, groupId)
        return null
    }

    /**
     * This pilot with the group to offer first: `groupId` if they may still
     * fly for it, else the controller's group if they may, else their first
     * own group, else the first they may fly for.
     */
    fun choiceFor(pilot: Pilot, groupId: String?): Choice {
        if (groups.isEmpty()) return Choice(pilot, null)
        val all = groupsFor(pilot)
        val group = groupId?.takeIf { it in all }
            ?: controllerGroup?.takeIf { it in all }
            ?: all.firstOrNull()
        return Choice(pilot, group)
    }

    companion object {
        val EMPTY = PilotList(null, emptyList(), emptyList())

        /** From the server's answer to GET /api/controller/v1/pilots. */
        fun fromJson(json: JSONObject): PilotList {
            val groups = json.getJSONArray("groups")
            val pilots = json.getJSONArray("pilots")
            return PilotList(
                controllerGroup = if (json.isNull("controller_group")) null else json.getString("controller_group"),
                groups = (0 until groups.length()).map { i ->
                    val group = groups.getJSONObject(i)
                    Group(group.getString("id"), group.getString("name"))
                },
                pilots = (0 until pilots.length()).map { i ->
                    val pilot = pilots.getJSONObject(i)
                    Pilot(
                        pilot.getString("id"),
                        pilot.getString("name"),
                        strings(pilot.getJSONArray("groups")),
                        strings(pilot.getJSONArray("other_groups")),
                    )
                },
            )
        }

        private fun strings(array: JSONArray) = (0 until array.length()).map { array.getString(it) }
    }

    fun toJson(): JSONObject = JSONObject()
        .put("controller_group", controllerGroup ?: JSONObject.NULL)
        .put("groups", JSONArray(groups.map { JSONObject().put("id", it.id).put("name", it.name) }))
        .put(
            "pilots",
            JSONArray(
                pilots.map {
                    JSONObject().put("id", it.id).put("name", it.name)
                        .put("groups", JSONArray(it.groups)).put("other_groups", JSONArray(it.otherGroups))
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

    /**
     * The list as last downloaded; empty before the first download. Kept under
     * a key of its own per API version, so an update doesn't read the older
     * app's list (it lists pilots per team) and downloads it anew.
     */
    var current: PilotList
        get() = list.getString(LIST_V1, null)?.let { PilotList.fromJson(JSONObject(it)) }
            ?: PilotList.EMPTY
        set(value) = list.edit().putString(LIST_V1, value.toJson().toString()).remove("list").apply()

    /** Records a choice, as the pilot's id and group; `null` is "Not in the list". */
    fun choose(choice: Choice?, now: Long = System.currentTimeMillis()) {
        choices.edit().putString(now.toString(), choice?.key ?: NOT_IN_THE_LIST).apply()
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
        val (pilotId, groupId) = parse(value)
        return Chosen(pilotId, groupId)
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

    /**
     * The pilot chosen last, if they can still be chosen, with their group,
     * or another to offer first if they can't fly for that one any more.
     */
    fun last(): Choice? {
        val value = choices.all.maxByOrNull { (time, _) -> time.toLong() }?.value as String?
        if (value == null || value == NOT_IN_THE_LIST) return null
        val (pilotId, groupId) = parse(value)
        val list = current
        return list.pilots.find { it.id == pilotId }?.let { list.choiceFor(it, groupId) }
    }

    private companion object {
        const val NOT_IN_THE_LIST = ""
        const val LIST_V1 = "list_v1"

        /**
         * A stored choice: the pilot's id and the group's, which the app
         * before groups (0.3.0) stored as its team, with the same id. Before
         * that, the id alone.
         */
        fun parse(value: String): Pair<String, String?> {
            val parts = value.split(':')
            return parts[0] to parts.getOrNull(1)?.ifEmpty { null }
        }
        /** A choice counts for logs written within this time after it. */
        const val EXPIRES_MS = 6 * 3_600_000L
    }
}
