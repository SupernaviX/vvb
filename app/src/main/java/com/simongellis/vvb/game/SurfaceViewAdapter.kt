package com.simongellis.vvb.game

import com.simongellis.vvb.emulator.Renderer

interface SurfaceViewAdapter {
    fun setRenderer(renderer: Renderer)
    fun onPause()
    fun onResume()
}