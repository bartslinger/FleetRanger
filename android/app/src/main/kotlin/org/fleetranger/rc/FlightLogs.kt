package org.fleetranger.rc

import android.content.Context
import android.os.Environment
import java.io.File

/**
 * DJI Pilot 2's flight logs on this controller, and where each stands: the
 * upload service and the logs screen both read them from here.
 */
class FlightLogs(context: Context) {
    private val sync = SyncSettings(context)
    /**
     * Logs the server has, by name, size and modification time: uploaded, or
     * found there already. Only for the install the controller is paired with.
     */
    private val uploaded = context.getSharedPreferences("uploaded", Context.MODE_PRIVATE)
    /** Folders a search found outside DJI's usual places, and whether it found nothing. */
    private val searched = context.getSharedPreferences("log_folders", Context.MODE_PRIVATE)

    enum class State { Uploaded, Waiting, BeingWritten, BeforeSyncDate }

    data class Log(val file: File, val state: State)

    /**
     * Whether the app gets at the logs: [Found] a FlightRecord folder it can
     * read; [NotFound] none, as on a controller that never flew, or after DJI
     * moved it; [Unreadable] a folder it may not read, so logs won't come.
     */
    enum class Access { Found, NotFound, Unreadable }

    /**
     * See [check]: the access, whether a search found nothing, and the state of
     * each place looked in (folder names only, never log names).
     */
    data class Check(val access: Access, val searchedInVain: Boolean, val places: Map<String, String>)

    /** Every log, newest first. */
    fun all(now: Long = System.currentTimeMillis()): List<Log> =
        folders()
            .flatMap { it.listFiles { file -> file.isFile && file.name.endsWith(".txt") }.orEmpty().toList() }
            .sortedByDescending { it.lastModified() }
            .map { file ->
                val state = when {
                    uploaded.contains(key(file)) -> State.Uploaded
                    !sync.includes(file.lastModified()) -> State.BeforeSyncDate
                    now - file.lastModified() < SETTLE_MS -> State.BeingWritten
                    else -> State.Waiting
                }
                Log(file, state)
            }

    fun markUploaded(key: String) = uploaded.edit().putBoolean(key, true).apply()

    /** Pairing (again) asks the server about every log: it may have some, all or none. */
    fun forgetUploaded() = uploaded.edit().clear().apply()

    /** Whether the app gets at the logs; for the notices and the error reports. */
    fun check(): Check {
        val storage = Environment.getExternalStorageDirectory()
        val found = folders()
        val places = (DJI_ROOTS + remembered()).associateWith { place ->
            val folder = File(storage, place)
            when {
                !folder.exists() -> "missing"
                folder.listFiles() == null -> "can't be read"
                else -> "readable"
            }
        } + ("storage" to Environment.getExternalStorageState())
        val access = when {
            found.any { it.listFiles() == null } || places.containsValue("can't be read") -> Access.Unreadable
            found.isEmpty() -> Access.NotFound
            else -> Access.Found
        }
        if (access == Access.Found) searched.edit().remove("in_vain").apply()
        return Check(access, searched.getBoolean("in_vain", false), places)
    }

    /**
     * Searches the shared storage for a FlightRecord folder with DJI's logs,
     * when the usual places have none, and uses it from then on. Returns its
     * path on the shared storage; null if there's none. Blocks, for seconds.
     */
    fun search(): String? {
        val storage = Environment.getExternalStorageDirectory()
        val folder = storage.walkTopDown()
            .maxDepth(SEARCH_DEPTH)
            .firstOrNull { it.isDirectory && it.name == "FlightRecord" && hasLogs(it) }
        if (folder == null) {
            searched.edit().putBoolean("in_vain", true).apply()
            return null
        }
        val path = folder.relativeTo(storage).path
        searched.edit()
            .putStringSet("folders", remembered() + path)
            .remove("in_vain")
            .apply()
        return path
    }

    private fun remembered(): Set<String> = searched.getStringSet("folders", null).orEmpty()

    /**
     * DJI Pilot 2 keeps its logs in a folder named FlightRecord: on the RC
     * Pro Enterprise in DJI/com.dji.industry.pilot on the shared storage.
     * Looking for the folder also finds it if a version moves it nearby;
     * folders a search found elsewhere count too.
     */
    private fun folders(): List<File> {
        val storage = Environment.getExternalStorageDirectory()
        val usual = DJI_ROOTS.map { File(storage, it) }
            .flatMap { root ->
                root.walkTopDown()
                    .maxDepth(4)
                    .filter { it.isDirectory && it.name == "FlightRecord" }
                    .toList()
            }
        return (usual + remembered().map { File(storage, it) }.filter { it.isDirectory }).distinct()
    }

    companion object {
        /** A log must be unchanged this long: DJI Pilot 2 may still be writing it. */
        const val SETTLE_MS = 3 * 60_000L

        /** How deep a search looks into the shared storage, my choice. */
        private const val SEARCH_DEPTH = 6

        /** Changes with the log, so a log that changes after its upload goes up again. */
        fun key(file: File) = "${file.name}|${file.length()}|${file.lastModified()}"

        private fun hasLogs(folder: File) =
            folder.listFiles { file -> file.isFile && file.name.startsWith("DJIFlightRecord_") }
                ?.isNotEmpty() == true

        private val DJI_ROOTS = listOf(
            "DJI/com.dji.industry.pilot",
            "Android/data/com.dji.industry.pilot/files",
        )
    }
}
