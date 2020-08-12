package com.simongellis.vvb

import android.content.res.Resources
import android.graphics.BitmapFactory
import java.nio.ByteBuffer

class Emulator(resources: Resources) {
    private var _pointer = 0L
    private var _resources = resources

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

    fun loadImage() {
        nativeLoadImage(
            loadResource(R.drawable.vbtitlescreen_left),
            loadResource(R.drawable.vbtitlescreen_right)
        )
    }

    private fun loadResource(id: Int): ByteBuffer {
        val options = BitmapFactory.Options()
        options.inScaled = false
        val bmp = BitmapFactory.decodeResource(_resources, id, options)
        val buffer = ByteBuffer.allocateDirect(bmp.byteCount)
        bmp.copyPixelsToBuffer(buffer)
        buffer.rewind()
        return buffer
    }

    private external fun nativeConstructor()
    private external fun nativeDestructor()
    private external fun nativeLoadImage(leftEye: ByteBuffer, rightEye: ByteBuffer)
}