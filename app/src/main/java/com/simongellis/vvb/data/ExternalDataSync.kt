package com.simongellis.vvb.data

import android.content.Context
import java.io.File
import java.io.InputStream
import java.io.OutputStream
import java.nio.ByteBuffer
import java.util.zip.ZipEntry
import java.util.zip.ZipInputStream
import java.util.zip.ZipOutputStream

class ExternalDataSync(private val context: Context) {
    fun export(stream: OutputStream) {
        ZipOutputStream(stream).use { zip ->
            zip.putNextEntry(ZipEntry("__version"))
            zip.write(1)
            zip.closeEntry()

            val gameDataDir = context.filesDir.resolve("game_data")
            exportFile(zip, gameDataDir, "game_data")
            zip.close()
        }
    }

    private fun exportFile(zip: ZipOutputStream, file: File, path: String) {
        if (file.isDirectory) {
            zip.putNextEntry(ZipEntry("$path/"))
            zip.closeEntry()
            for (child in file.listFiles() ?: arrayOf()) {
                exportFile(zip, child, "$path/${child.name}")
            }
        } else {
            zip.putNextEntry(ZipEntry(path))
            file.inputStream().use { it.pipe(zip) }
            zip.closeEntry()
        }
    }

    fun import(stream: InputStream): Result<Unit> {
        val filesDir = context.filesDir
        ZipInputStream(stream).use { zip ->
            val versionEntry = zip.nextEntry
            if (versionEntry?.name != "__version" || versionEntry.isDirectory) {
                return Result.failure(IllegalArgumentException("No version found in backup"))
            }
            val version = zip.read()
            if (version != 1) {
                return Result.failure(IllegalArgumentException("Unknown version $version"))
            }

            var entry = zip.nextEntry
            while (entry != null) {
                val file = filesDir.resolve(entry.name)
                if (entry.isDirectory) {
                    file.mkdirs()
                } else {
                    file.outputStream().use { zip.pipe(it) }
                }
                entry = zip.nextEntry
            }
        }
        return Result.success(Unit)
    }

    private fun InputStream.pipe(out: OutputStream) {
        val buffer = ByteArray(8192)
        var read: Int
        while (true) {
            read = this.read(buffer, 0, 8192)
            if (read < 0) {
                return
            }
            out.write(buffer, 0, read)
        }
    }
}