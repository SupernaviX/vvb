package com.simongellis.vvb

import android.content.pm.ActivityInfo
import android.opengl.GLSurfaceView
import android.os.Bundle
import androidx.appcompat.app.AppCompatActivity
import com.simongellis.vvb.databinding.ActivityGameBinding

class VideoPreviewActivity: AppCompatActivity() {
    private lateinit var _binding: ActivityGameBinding
    private lateinit var _emulator: Emulator
    private lateinit var _renderer: Renderer

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        _binding = ActivityGameBinding.inflate(layoutInflater)
        _emulator = Emulator.getInstance(applicationContext)
        _renderer = Renderer(_emulator, Settings(applicationContext))

        requestedOrientation = ActivityInfo.SCREEN_ORIENTATION_LANDSCAPE
        setContentView(_binding.root)

        val surfaceView = _binding.surfaceView
        surfaceView.setEGLContextClientVersion(2)
        surfaceView.setRenderer(_renderer)
        surfaceView.renderMode = GLSurfaceView.RENDERMODE_CONTINUOUSLY

        _emulator.loadImage()
    }

    override fun onResume() {
        super.onResume()
        _binding.surfaceView.onResume()
        _renderer.ensureDeviceParams()
    }

    override fun onPause() {
        super.onPause()
        _binding.surfaceView.onPause()
    }

    override fun onDestroy() {
        super.onDestroy()
        _renderer.destroy()
    }
}