package com.simongellis.vvb.emulator

interface Renderer {
    fun onSurfaceCreated()
    fun onSurfaceChanged(width: Int, height: Int)
    fun onDrawFrame()
    fun destroy()
    fun onResume() {}
    fun onModeChanged(enable3d: Boolean) {}
}