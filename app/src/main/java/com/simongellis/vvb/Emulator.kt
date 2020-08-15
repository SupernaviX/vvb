package com.simongellis.vvb

import android.content.Context
import android.graphics.BitmapFactory
import android.net.Uri
import java.nio.ByteBuffer

class Emulator(context: Context) {
    private var _pointer = 0L
    private var _context = context

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
        nativeLoadGamePakRom(rom)
    }

    fun run() {
        nativeRun()
    }

    fun loadImage() {
        nativeLoadImage(
            loadResource(R.drawable.vbtitlescreen_left),
            loadResource(R.drawable.vbtitlescreen_right)
        )
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
    private external fun nativeRun()
    private external fun nativeLoadImage(leftEye: ByteBuffer, rightEye: ByteBuffer)
}