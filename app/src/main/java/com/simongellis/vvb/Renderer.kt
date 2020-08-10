package com.simongellis.vvb

import android.content.res.Resources
import android.graphics.BitmapFactory
import android.opengl.GLSurfaceView
import java.nio.ByteBuffer
import javax.microedition.khronos.egl.EGLConfig
import javax.microedition.khronos.opengles.GL10

class Renderer(resources: Resources) : GLSurfaceView.Renderer {
    private var _pointer = 0L
    private var _resources = resources

    init {
        nativeConstructor()
    }

    fun destroy() {
        if (_pointer != 0L) {
            nativeDestructor()
        }
    }

    override fun onSurfaceCreated(gl: GL10?, config: EGLConfig?) {
        nativeOnSurfaceCreated(loadTitleScreen())
    }

    override fun onSurfaceChanged(gl: GL10?, width: Int, height: Int) {
        nativeOnSurfaceChanged(width, height)
    }

    override fun onDrawFrame(gl: GL10?) {
        nativeOnDrawFrame()
    }

    fun ensureDeviceParams() {
        nativeEnsureDeviceParams()
    }

    fun changeDeviceParams() {
        nativeChangeDeviceParams()
    }

    private fun loadTitleScreen(): ByteBuffer {
        val options = BitmapFactory.Options()
        options.inScaled = false
        val bmp = BitmapFactory.decodeResource(_resources, R.drawable.vbtitlescreen, options)
        val buffer = ByteBuffer.allocateDirect(bmp.byteCount)
        bmp.copyPixelsToBuffer(buffer)
        buffer.rewind()
        return buffer
    }

    private external fun nativeConstructor()
    private external fun nativeDestructor()
    private external fun nativeOnSurfaceCreated(titleScreen: ByteBuffer)
    private external fun nativeOnSurfaceChanged(width: Int, height: Int)
    private external fun nativeOnDrawFrame()
    private external fun nativeEnsureDeviceParams()
    private external fun nativeChangeDeviceParams()
}