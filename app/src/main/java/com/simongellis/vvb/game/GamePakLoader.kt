package com.simongellis.vvb.game

import android.content.Context
import android.net.Uri
import androidx.annotation.StringRes
import com.simongellis.vvb.R
import com.simongellis.vvb.emulator.GamePak
import java.io.File
import java.io.FileNotFoundException
import java.math.BigInteger
import java.security.MessageDigest
import java.util.zip.ZipInputStream

class GamePakLoader(private val context: Context) {
    fun tryLoad(uri: Uri): Result<GamePak> {
        return Result.runCatching { load(uri) }
    }
    fun load(uri: Uri): GamePak {
        val ext = tryGetExtension(uri)
        val rom = try {
            when {
                ext == "zip" -> loadZipFile(uri)
                isVbFileExt(ext) -> loadVbFile(uri)
                hasZipHeader(uri) -> loadZipFile(uri)
                else -> loadVbFile(uri)
            }
        } catch (ex: FileNotFoundException) {
            throw error(R.string.error_file_not_found)
        }
        val hash = hashRom(rom)
        val gameDataDir = File(context.filesDir, "game_data/${hash}")
        return GamePak(rom, hash, gameDataDir)
    }

    private fun loadVbFile(uri: Uri): ByteArray {
        val size = context.contentResolver.openAssetFileDescriptor(uri, "r")
            ?.use { it.length }
            ?: throw error(R.string.error_file_not_found)
        if (size.countOneBits() != 1) {
            throw error(R.string.error_not_power_of_two)
        }
        if (size > 0x01000000) {
            throw error(R.string.error_too_large)
        }

        return context.contentResolver.openInputStream(uri)
            ?.use { it.readBytes() }
            ?: throw error(R.string.error_file_not_found)
    }

    private fun loadZipFile(uri: Uri): ByteArray {
        val inputStream = context.contentResolver.openInputStream(uri)
        ZipInputStream(inputStream).use { zip ->
            for (entry in generateSequence { zip.nextEntry }) {
                val ext = tryGetExtension(entry.name)
                val size = entry.size
                if (isVbFileExt(ext) && size.countOneBits() == 1 && size <= 0x01000000) {
                    return zip.readBytes()
                }
            }
        }
        throw error(R.string.error_zip)
    }

    private fun hasZipHeader(uri: Uri): Boolean {
        val header = context.contentResolver.openInputStream(uri)?.use {
            val buffer = ByteArray(4)
            var bytesCopied = 0
            var bytes = it.read(buffer)
            while (bytes >= 0 && bytesCopied < buffer.size) {
                bytesCopied += bytes
                bytes = it.read(buffer, bytesCopied, buffer.size - bytesCopied)
            }
            buffer
        } ?: throw error(R.string.error_file_not_found)
        val zipHeader = byteArrayOf(0x50, 0x4b, 0x03, 0x04)
        return zipHeader.zip(header).all { it.first == it.second }
    }

    private fun error(@StringRes message: Int): IllegalArgumentException {
        return IllegalArgumentException(context.getString(message))
    }

    private fun tryGetExtension(uri: Uri): String? {
        return uri.path?.let { tryGetExtension(it) }
    }

    private fun tryGetExtension(path: String): String? {
        val sep = path.lastIndexOf('.')
        return if (sep == -1) {
            null
        } else {
            path.substring(sep + 1).lowercase()
        }
    }

    private fun isVbFileExt(ext: String?): Boolean {
        return ext == "vb" || ext == "vboy" || ext == "bin"
    }

    private fun hashRom(rom: ByteArray): String {
        return MessageDigest.getInstance("MD5").digest(rom).toHexString()
    }

    private fun ByteArray.toHexString(): String {
        return BigInteger(1, this).toString(16).padStart(32, '0')
    }
}