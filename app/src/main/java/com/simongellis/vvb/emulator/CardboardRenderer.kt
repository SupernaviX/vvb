package com.simongellis.vvb.emulator

import javax.microedition.khronos.egl.EGLConfig
import javax.microedition.khronos.opengles.GL10

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

    override fun onSurfaceCreated(gl: GL10?, config: EGLConfig?) {
        nativeOnSurfaceCreated()
    }

    override fun onSurfaceChanged(gl: GL10?, width: Int, height: Int) {
        nativeOnSurfaceChanged(width, height)
    }

    override fun onDrawFrame(gl: GL10?) {
        nativeOnDrawFrame()
    }

    override fun onResume() {
        nativeEnsureDeviceParams()
    }

    private external fun nativeConstructor(emulator: Emulator, settings: Settings)
    private external fun nativeDestructor()
    private external fun nativeOnSurfaceCreated()
    private external fun nativeOnSurfaceChanged(width: Int, height: Int)
    private external fun nativeOnDrawFrame()
    private external fun nativeEnsureDeviceParams()
}