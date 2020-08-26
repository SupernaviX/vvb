package com.simongellis.vvb

import android.content.pm.ActivityInfo
import android.opengl.GLSurfaceView
import androidx.appcompat.app.AppCompatActivity
import android.os.Bundle
import kotlinx.android.synthetic.main.activity_game.*

class GameActivity : AppCompatActivity() {
    private lateinit var _emulator: Emulator
    private lateinit var _renderer: Renderer

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        _emulator = Emulator.getInstance(applicationContext)
        _renderer = Renderer(_emulator)

        requestedOrientation = ActivityInfo.SCREEN_ORIENTATION_LANDSCAPE
        setContentView(R.layout.activity_game)

        surface_view.setEGLContextClientVersion(2)
        surface_view.setRenderer(_renderer)
        surface_view.renderMode = GLSurfaceView.RENDERMODE_CONTINUOUSLY

        _emulator.loadImage()
    }

    override fun onPause() {
        super.onPause()
        surface_view.onPause()
        _emulator.pause()
    }

    override fun onResume() {
        super.onResume()
        surface_view.onResume()
        _renderer.ensureDeviceParams()
        // _emulator.loadImage()
        _emulator.resume()
    }

    override fun onDestroy() {
        super.onDestroy()
        _renderer.destroy()
    }
}