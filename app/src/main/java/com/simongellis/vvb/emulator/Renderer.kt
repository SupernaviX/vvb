package com.simongellis.vvb.emulator

import android.opengl.GLSurfaceView

interface Renderer : GLSurfaceView.Renderer {
    fun destroy()
    fun onResume() {}
}