package org.fleetranger.rc

import android.content.Context
import java.io.File
import java.io.RandomAccessFile
import java.nio.ByteBuffer
import java.nio.ByteOrder

/**
 * The install's address, written into this app's APK when it was downloaded
 * from the install's download page: an ID-value pair in the APK Signing
 * Block, which Android's verifier ignores (backend/src/download/apk.rs).
 * Null if the APK has none, e.g. one installed by other means.
 */
object Setup {
    /** "FRG1", as in the backend. */
    private const val SERVER_ID = 0x46524731
    private const val EOCD_SIGNATURE = 0x06054b50
    private const val EOCD_SIZE = 22
    private val MAGIC = "APK Sig Block 42".toByteArray()

    fun server(context: Context): String? = try {
        read(File(context.applicationInfo.sourceDir))
            ?.takeIf { it.startsWith("https://") || (BuildConfig.DEBUG && it.startsWith("http://")) }
    } catch (_: Exception) {
        // A malformed or unexpected APK: the pairing screen asks instead.
        null
    }

    private fun read(apk: File): String? = RandomAccessFile(apk, "r").use { file ->
        // The end of central directory record, followed by a comment of up to 64 KiB.
        val tailSize = minOf(file.length(), (EOCD_SIZE + 0xffff).toLong()).toInt()
        val tail = ByteArray(tailSize)
        file.seek(file.length() - tailSize)
        file.readFully(tail)
        val tailBuffer = ByteBuffer.wrap(tail).order(ByteOrder.LITTLE_ENDIAN)
        val eocd = (tailSize - EOCD_SIZE downTo 0).firstOrNull { at ->
            tailBuffer.getInt(at) == EOCD_SIGNATURE &&
                at + EOCD_SIZE + (tailBuffer.getShort(at + 20).toInt() and 0xffff) == tailSize
        } ?: return null
        val cdOffset = tailBuffer.getInt(eocd + 16).toLong() and 0xffffffffL

        // The block ends with its size and the magic, right before the central directory.
        val footer = ByteArray(24)
        file.seek(cdOffset - 24)
        file.readFully(footer)
        if (!footer.copyOfRange(8, 24).contentEquals(MAGIC)) return null
        val size = ByteBuffer.wrap(footer).order(ByteOrder.LITTLE_ENDIAN).getLong(0)
        if (size < 24 || size > cdOffset) return null
        // The size counts everything after the first size field: pairs, size, magic.
        val pairs = ByteArray((size - 24).toInt())
        file.seek(cdOffset - size)
        file.readFully(pairs)

        val buffer = ByteBuffer.wrap(pairs).order(ByteOrder.LITTLE_ENDIAN)
        while (buffer.remaining() >= 12) {
            val length = buffer.getLong()
            if (length < 4 || length > buffer.remaining()) return null
            val id = buffer.getInt()
            val value = ByteArray((length - 4).toInt())
            buffer.get(value)
            if (id == SERVER_ID) return String(value, Charsets.UTF_8)
        }
        null
    }
}
