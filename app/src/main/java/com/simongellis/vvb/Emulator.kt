package com.simongellis.vvb

import android.content.Context
import android.graphics.BitmapFactory
import android.net.Uri
import android.os.SystemClock
import java.io.File
import java.nio.ByteBuffer
import java.util.*
import kotlin.concurrent.thread

class Emulator(context: Context) {
    private var _pointer = 0L
    private var _context = context
    private var _thread: Thread? = null
    private var _running = false
    private var _gameLoaded = false

    private var _sram: File? = null
    private var _sramBuffer = ByteBuffer.allocateDirect(8 * 1024)

    private var _activeInputs = 0

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

    fun loadGamePak(romUri: Uri) {
        pause()
        val rom = loadFile(romUri)

        val sram = File(_context.filesDir, getSRAMFilename(romUri))
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

    fun loadImage() {
        nativeLoadImage(
            loadResource(R.drawable.vbtitlescreen_left),
            loadResource(R.drawable.vbtitlescreen_right)
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
        _thread = thread(name = "EmulatorThread") { run() }
    }

    fun pause() {
        if (!_running) {
            return
        }
        _running = false
        _thread?.join()
        saveSRAM()
        _activeInputs = 0
    }

    fun keyDown(input: Input) {
        _activeInputs = _activeInputs or input.bitMask
    }

    fun keyUp(input: Input) {
        _activeInputs = _activeInputs and input.bitMask.inv()
    }

    private fun run() {
        var then = SystemClock.elapsedRealtimeNanos()
        while (_running) {
            val now = SystemClock.elapsedRealtimeNanos()
            nativeTick((now - then).toInt(), _activeInputs)
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

    private fun saveSRAM() {
        val sram = _sram ?: return
        _sramBuffer.rewind()
        nativeReadSRAM(_sramBuffer)
        _sramBuffer.rewind()
        sram.outputStream().channel.use { it.write(_sramBuffer) }
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
    private external fun nativeLoadGamePak(rom: ByteBuffer, sram: ByteBuffer)
    private external fun nativeTick(nanoseconds: Int, inputMask: Int)
    private external fun nativeReadSRAM(buffer: ByteBuffer)
    private external fun nativeLoadImage(leftEye: ByteBuffer, rightEye: ByteBuffer)

    companion object {
        @Volatile
        private var INSTANCE: Emulator? = null

        fun getInstance(context: Context): Emulator = INSTANCE ?: synchronized(this) {
            INSTANCE ?: Emulator(context).also { INSTANCE = it }
        }
    }
}
