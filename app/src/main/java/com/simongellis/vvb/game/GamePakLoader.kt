package com.simongellis.vvb.game

import android.content.Context
import android.net.Uri
import androidx.annotation.StringRes
import com.simongellis.vvb.R
import com.simongellis.vvb.emulator.GamePak
import java.io.File
import java.io.FileNotFoundException
import java.util.zip.ZipInputStream

class GamePakLoader(private val context: Context) {
    fun tryLoad(uri: Uri): GamePak {
        val (name, ext) = getNameAndExt(uri)
        val rom = try {
            when(ext) {
                "vb" -> loadVbFile(uri)
                "zip" -> loadZipFile(uri)
                else -> throw error(R.string.error_unrecognized_extension)
            }
        } catch (ex: FileNotFoundException) {
            throw error(R.string.error_file_not_found)
        }
        val sram = File(context.filesDir, "${name}.srm")
        return GamePak(name, rom, sram)
    }

    private fun loadVbFile(uri: Uri): ByteArray {
        val size = context.contentResolver.openAssetFileDescriptor(uri, "r")!!.use {
            it.length
        }
        if (size.countOneBits() != 1) {
            throw error(R.string.error_not_power_of_two)
        }
        if (size > 0x01000000) {
            throw error(R.string.error_too_large)
        }

        context.contentResolver.openInputStream(uri)!!.use { inputStream ->
            return inputStream.readBytes()
        }
    }

    private fun loadZipFile(uri: Uri): ByteArray {
        val inputStream = context.contentResolver.openInputStream(uri)
        ZipInputStream(inputStream).use { zip ->
            for (entry in generateSequence { zip.nextEntry }) {
                val (_, ext) = getNameAndExt(entry.name)
                val size = entry.size
                if (ext == "vb" && size.countOneBits() == 1 && size <= 0x01000000) {
                    return zip.readBytes()
                }
            }
        }
        throw IllegalArgumentException(context.getString(R.string.error_zip))
    }

    private fun error(@StringRes message: Int): IllegalArgumentException {
        return IllegalArgumentException(context.getString(message))
    }

    companion object {
        fun getName(uri: Uri): String {
            val (name) = getNameAndExt(uri)
            return name
        }

        private fun getNameAndExt(uri: Uri): Pair<String, String> {
            val path = uri.lastPathSegment!!
            return getNameAndExt(path)
        }

        private fun getNameAndExt(path: String): Pair<String, String> {
            val filename = path.substringAfterLast('/')
            val sep = filename.lastIndexOf('.')
            return filename.substring(0, sep) to filename.substring(sep + 1)
        }
    }
}