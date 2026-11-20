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

    enum class State { Uploaded, Waiting, BeingWritten, BeforeSyncDate }

    data class Log(val file: File, val state: State)

    /** See [folderProblem]: what's wrong, and the state of each place looked in. */
    data class FolderProblem(val message: String, val places: Map<String, String>)

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

    companion object {
        /** A log must be unchanged this long: DJI Pilot 2 may still be writing it. */
        const val SETTLE_MS = 3 * 60_000L

        /**
         * What keeps the app from DJI Pilot 2's logs, for the error reports:
         * no FlightRecord folder in the usual places, or one it can't read;
         * null when there's none. Holds folder names only, never log names.
         */
        fun folderProblem(): FolderProblem? {
            val found = folders()
            val unreadable = found.filter { it.listFiles() == null }
            val message = when {
                found.isEmpty() -> "No FlightRecord folder found"
                unreadable.isNotEmpty() -> "A FlightRecord folder can't be read"
                else -> return null
            }
            val storage = Environment.getExternalStorageDirectory()
            val places = DJI_ROOTS.associateWith { root ->
                val folder = File(storage, root)
                when {
                    !folder.exists() -> "missing"
                    folder.listFiles() == null -> "can't be read"
                    else -> "readable"
                }
            } + unreadable.associate { it.relativeTo(storage).path to "can't be read" }
            return FolderProblem(message, places + ("storage" to Environment.getExternalStorageState()))
        }


        /** Changes with the log, so a log that changes after its upload goes up again. */
        fun key(file: File) = "${file.name}|${file.length()}|${file.lastModified()}"

        /**
         * DJI Pilot 2 keeps its logs in a folder named FlightRecord: on the RC
         * Pro Enterprise in DJI/com.dji.industry.pilot on the shared storage.
         * Looking for the folder also finds it if a version moves it.
         */
        private fun folders(): List<File> =
            DJI_ROOTS.map { File(Environment.getExternalStorageDirectory(), it) }
                .flatMap { root ->
                    root.walkTopDown()
                        .maxDepth(4)
                        .filter { it.isDirectory && it.name == "FlightRecord" }
                        .toList()
                }
                .distinct()

        private val DJI_ROOTS = listOf(
            "DJI/com.dji.industry.pilot",
            "Android/data/com.dji.industry.pilot/files",
        )
    }
}
