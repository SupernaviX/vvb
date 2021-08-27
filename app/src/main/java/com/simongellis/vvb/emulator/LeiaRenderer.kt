package com.simongellis.vvb.emulator

import androidx.annotation.ColorInt
import javax.microedition.khronos.egl.EGLConfig
import javax.microedition.khronos.opengles.GL10

class LeiaRenderer(emulator: Emulator, settings: Settings) : Renderer {
    private var _pointer = 0L

    init {
        nativeConstructor(emulator, settings)
    }

    fun finalize() {
        destroy()
    }

    override fun destroy() {
        if (_pointer != 0L) {
            nativeDestructor()
        }
    }

    override fun onSurfaceCreated(gl: GL10?, config: EGLConfig?) {
        nativeOnSurfaceCreated()
    }

    override fun onSurfaceChanged(gl: GL10?, width: Int, height: Int) {
        nativeOnSurfaceChanged(width, height)
    }

    override fun onDrawFrame(gl: GL10?) {
        nativeOnDrawFrame()
    }

    class Settings(
        val screenZoom: Float,
        val verticalOffset: Float,
        @ColorInt val color: Int,
        @ColorInt val colorBG: Int)

    private external fun nativeConstructor(emulator: Emulator, settings: Settings)
    private external fun nativeDestructor()
    private external fun nativeOnSurfaceCreated()
    private external fun nativeOnSurfaceChanged(width: Int, height: Int)
    private external fun nativeOnDrawFrame()
}