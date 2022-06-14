package com.simongellis.vvb.emulator

import android.graphics.Bitmap
import android.os.SystemClock
import java.io.File
import java.nio.ByteBuffer
import kotlin.concurrent.thread

class Emulator {
    private var _pointer = 0L
    private var _thread: Thread? = null
    private var _running = false
    private var _gamePak: GamePak? = null
    private var _autoSave: File? = null

    private val _sramBuffer = ByteBuffer.allocateDirect(GamePak.sramSize)

    init {
        nativeConstructor()
    }

    fun finalize() {
        destroy()
    }

    private fun destroy() {
        if (_pointer != 0L) {
            nativeDestructor()
        }
    }

    fun loadGamePak(gamePak: GamePak) {
        pause()

        val rom = ByteBuffer.allocateDirect(gamePak.rom.size)
        rom.put(gamePak.rom)
        rom.rewind()

        _sramBuffer.rewind()
        gamePak.loadSram(_sramBuffer)
        _sramBuffer.rewind()

        nativeLoadGamePak(rom, _sramBuffer)

        _gamePak = gamePak
        _autoSave = null
    }

    fun setAutoSaveFile(file: File?) {
        _autoSave = file
    }

    fun unloadGamePak() {
        pause()
        nativeUnloadGamePak()
        _gamePak = null
        _autoSave = null
    }

    fun saveState(state: File) {
        nativeSaveState(state.canonicalPath)
    }

    fun loadState(state: File) {
        nativeLoadState(state.canonicalPath)
    }

    fun reset() {
        pause()
        nativeReset()
    }

    fun loadImage(leftEye: Bitmap, rightEye: Bitmap) {
        nativeLoadImage(leftEye.toByteBuffer(), rightEye.toByteBuffer())
    }

    fun resume() {
        if (_gamePak == null) {
            return
        }
        _running = true
        _thread = thread(name = "EmulatorThread", priority = -12) { run() }
    }

    fun performAutoSave(): Boolean {
        _autoSave?.also(this::saveState)
        return _autoSave != null
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
        var lastDuration = DEFAULT_TICK_DURATION
        while (_running) {
            val start = SystemClock.elapsedRealtimeNanos()
            nativeTick(lastDuration.toInt())
            val duration = SystemClock.elapsedRealtimeNanos() - start
            lastDuration = duration.coerceAtMost(MAX_TICK_DURATION)
        }
    }

    private fun saveSRAM() {
        val gamePak = _gamePak ?: return
        _sramBuffer.rewind()
        nativeReadSRAM(_sramBuffer)
        _sramBuffer.rewind()
        gamePak.saveSram(_sramBuffer)
    }

    private fun Bitmap.toByteBuffer(): ByteBuffer {
        val buffer = ByteBuffer.allocateDirect(byteCount)
        copyPixelsToBuffer(buffer)
        buffer.rewind()
        return buffer
    }

    private external fun nativeConstructor()
    private external fun nativeDestructor()
    private external fun nativeLoadGamePak(rom: ByteBuffer, sram: ByteBuffer)
    private external fun nativeUnloadGamePak()
    private external fun nativeSaveState(path: String)
    private external fun nativeLoadState(path: String)
    private external fun nativeReset()
    private external fun nativeTick(nanoseconds: Int)
    private external fun nativeReadSRAM(buffer: ByteBuffer)
    private external fun nativeLoadImage(leftEye: ByteBuffer, rightEye: ByteBuffer)

    companion object {
        val instance: Emulator by lazy { Emulator() }
        private const val DEFAULT_TICK_DURATION = 5_000_000L
        private const val MAX_TICK_DURATION = 1_000_000_000L
    }
}
