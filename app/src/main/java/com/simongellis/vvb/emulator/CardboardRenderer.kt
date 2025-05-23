package com.simongellis.vvb.emulator

import androidx.annotation.ColorInt

class CardboardRenderer(emulator: Emulator, settings: Settings) : Renderer {
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

    override fun onSurfaceCreated() {
        nativeOnSurfaceCreated()
    }

    override fun onSurfaceChanged(width: Int, height: Int) {
        nativeOnSurfaceChanged(width, height)
    }

    override fun onDrawFrame() {
        nativeOnDrawFrame()
    }

    override fun onResume() {
        nativeEnsureDeviceParams()
    }

    class Settings(
        val screenZoom: Float,
        val aspectRatio: Int,
        val verticalOffset: Float,
        @ColorInt val color: Int)

    private external fun nativeConstructor(emulator: Emulator, settings: Settings)
    private external fun nativeDestructor()
    private external fun nativeOnSurfaceCreated()
    private external fun nativeOnSurfaceChanged(width: Int, height: Int)
    private external fun nativeOnDrawFrame()
    private external fun nativeEnsureDeviceParams()
}