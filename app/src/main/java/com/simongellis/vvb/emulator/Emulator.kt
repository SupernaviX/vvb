package com.simongellis.vvb.emulator

import android.content.Context
import android.graphics.BitmapFactory
import android.net.Uri
import android.os.SystemClock
import com.simongellis.vvb.R
import java.io.File
import java.io.InputStream
import java.nio.ByteBuffer
import java.util.*
import java.util.zip.ZipInputStream
import kotlin.IllegalArgumentException
import kotlin.concurrent.thread

class Emulator {
    private var _pointer = 0L
    private var _thread: Thread? = null
    private var _running = false
    private var _gameLoaded = false

    private var _sram: File? = null
    private var _sramBuffer = ByteBuffer.allocateDirect(8 * 1024)

    init {
        nativeConstructor()
    }

    fun finalize() {
        destroy()
    }

    fun destroy() {
        if (_pointer != 0L) {
            nativeDestructor()
        }
    }

    fun tryLoadGamePak(context: Context, romUri: Uri) {
        pause()

        val (name, ext) = getNameAndExt(romUri)
        val rom = when(ext) {
            "vb" -> loadVbFile(context, romUri)
            "zip" -> loadZipFile(context, romUri)
            else -> throw IllegalArgumentException(context.getString(R.string.error_unrecognized_extension))
        }

        val sram = File(context.filesDir, "${name}.srm")
        _sram = sram

        if (sram.exists()) {
            _sramBuffer.rewind()
            _sramBuffer.put(sram.readBytes())
        } else {
            Arrays.fill(_sramBuffer.array(), 0)
        }

        _sramBuffer.rewind()
        nativeLoadGamePak(rom, _sramBuffer)

        _gameLoaded = true
    }

    fun loadImage(context: Context) {
        nativeLoadImage(
            loadResource(context, R.drawable.vbtitlescreen_left),
            loadResource(context, R.drawable.vbtitlescreen_right)
        )
    }

    fun isGameLoaded(): Boolean {
        return _gameLoaded
    }

    fun resume() {
        if (!_gameLoaded) {
            return
        }
        _running = true
        _thread = thread(name = "EmulatorThread", priority = -12) { run() }
    }

    fun pause() {
        if (!_running) {
            return
        }
        _running = false
        _thread?.join()
        saveSRAM()
    }

    private fun run() {
        var then = SystemClock.elapsedRealtimeNanos()
        while (_running) {
            val now = SystemClock.elapsedRealtimeNanos()
            // By default, emulate however much time passed since the last tick,
            // but cap it to 1 second in case of extreme lag
            val duration = kotlin.math.min((now - then).toInt(), 1_000_000_000)
            nativeTick(duration)
            then = now
        }
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


    private fun loadVbFile(context: Context, uri: Uri): ByteBuffer {
        val size = context.contentResolver.openAssetFileDescriptor(uri, "r")!!.use {
            it.length
        }
        if (size.countOneBits() != 1) {
            throw IllegalArgumentException(context.getString(R.string.error_not_power_of_two))
        }
        if (size > 0x01000000) {
            throw IllegalArgumentException(context.getString(R.string.error_too_large))
        }

        context.contentResolver.openInputStream(uri)!!.use { inputStream ->
            return inputStream.toByteBuffer()
        }
    }

    private fun loadZipFile(context: Context, uri: Uri): ByteBuffer {
        val inputStream = context.contentResolver.openInputStream(uri)
        ZipInputStream(inputStream).use { zip ->
            for (entry in generateSequence { zip.nextEntry }) {
                val (_, ext) = getNameAndExt(entry.name)
                val size = entry.size
                if (ext == "vb" && size.countOneBits() == 1 && size <= 0x01000000) {
                    return zip.toByteBuffer()
                }
            }
        }
        throw IllegalArgumentException(context.getString(R.string.error_zip))
    }

    private fun InputStream.toByteBuffer(): ByteBuffer {
        val bytes = readBytes()
        val buffer = ByteBuffer.allocateDirect(bytes.size)
        buffer.put(bytes)
        buffer.rewind()
        return buffer
    }

    private fun saveSRAM() {
        val sram = _sram ?: return
        _sramBuffer.rewind()
        nativeReadSRAM(_sramBuffer)
        _sramBuffer.rewind()
        sram.outputStream().channel.use { it.write(_sramBuffer) }
    }

    private fun loadResource(context: Context, id: Int): ByteBuffer {
        val options = BitmapFactory.Options()
        options.inScaled = false
        val bmp = BitmapFactory.decodeResource(context.resources, id, options)
        val buffer = ByteBuffer.allocateDirect(bmp.byteCount)
        bmp.copyPixelsToBuffer(buffer)
        buffer.rewind()
        return buffer
    }

    private external fun nativeConstructor()
    private external fun nativeDestructor()
    private external fun nativeLoadGamePak(rom: ByteBuffer, sram: ByteBuffer)
    private external fun nativeTick(nanoseconds: Int)
    private external fun nativeReadSRAM(buffer: ByteBuffer)
    private external fun nativeLoadImage(leftEye: ByteBuffer, rightEye: ByteBuffer)

    companion object {
        val instance: Emulator by lazy { Emulator() }
    }
}
