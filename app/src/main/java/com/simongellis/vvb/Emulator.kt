package com.simongellis.vvb

import android.content.Context
import android.graphics.BitmapFactory
import android.net.Uri
import android.os.SystemClock
import java.io.File
import java.nio.ByteBuffer
import java.util.*
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

    fun loadGamePak(context: Context, romUri: Uri) {
        pause()
        val rom = loadFile(context, romUri)

        val sram = File(context.filesDir, getSRAMFilename(romUri))
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

    private fun getSRAMFilename(romUri: Uri): String {
        val pathEnd = romUri.lastPathSegment!!
        return pathEnd.substringAfterLast('/').replace(Regex("\\.[^.]+$"), ".srm")
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

    private fun loadFile(context: Context, file: Uri): ByteBuffer {
        context.contentResolver.openInputStream(file)!!.use { inputStream ->
            val bytes = inputStream.readBytes()
            val rom = ByteBuffer.allocateDirect(bytes.size).put(bytes)
            rom.rewind()
            return rom
        }
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
        @Volatile
        private var INSTANCE: Emulator? = null

        fun getInstance(): Emulator = INSTANCE ?: synchronized(this) {
            INSTANCE ?: Emulator().also { INSTANCE = it }
        }
    }
}
