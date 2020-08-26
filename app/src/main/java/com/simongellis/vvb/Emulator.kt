package com.simongellis.vvb

import android.content.Context
import android.graphics.BitmapFactory
import android.net.Uri
import android.os.SystemClock
import java.nio.ByteBuffer
import kotlin.concurrent.thread

class Emulator(context: Context) {
    private var _pointer = 0L
    private var _context = context
    private var _thread: Thread? = null
    private var _running = false
    private var _gameLoaded = false

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

    fun loadGamePak(file: Uri) {
        val rom = loadFile(file)
        pause()
        nativeLoadGamePakRom(rom)
        _gameLoaded = true
    }

    fun loadImage() {
        nativeLoadImage(
            loadResource(R.drawable.vbtitlescreen_left),
            loadResource(R.drawable.vbtitlescreen_right)
        )
    }

    fun resume() {
        if (!_gameLoaded) {
            return
        }
        _running = true
        _thread = thread(name = "EmulatorThread") { run() }
    }

    fun pause() {
        _running = false
        _thread?.join()
    }

    private fun run() {
        var then = SystemClock.elapsedRealtimeNanos()
        while (_running) {
            val now = SystemClock.elapsedRealtimeNanos()
            nativeTick((now - then).toInt())
            then = now
        }
    }

    private fun loadFile(file: Uri): ByteBuffer {
        _context.contentResolver.openInputStream(file)!!.use { inputStream ->
            val bytes = inputStream.readBytes()
            val rom = ByteBuffer.allocateDirect(bytes.size).put(bytes)
            rom.rewind()
            return rom
        }
    }

    private fun loadResource(id: Int): ByteBuffer {
        val options = BitmapFactory.Options()
        options.inScaled = false
        val bmp = BitmapFactory.decodeResource(_context.resources, id, options)
        val buffer = ByteBuffer.allocateDirect(bmp.byteCount)
        bmp.copyPixelsToBuffer(buffer)
        buffer.rewind()
        return buffer
    }

    private external fun nativeConstructor()
    private external fun nativeDestructor()
    private external fun nativeLoadGamePakRom(rom: ByteBuffer)
    private external fun nativeTick(nanoseconds: Int)
    private external fun nativeLoadImage(leftEye: ByteBuffer, rightEye: ByteBuffer)

    companion object {
        @Volatile
        private var INSTANCE: Emulator? = null

        fun getInstance(context: Context): Emulator = INSTANCE ?: synchronized(this) {
            INSTANCE ?: Emulator(context).also { INSTANCE = it }
        }
    }
}