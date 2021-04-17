package com.simongellis.vvb

import android.content.pm.ActivityInfo
import android.opengl.GLSurfaceView
import android.os.Bundle
import androidx.appcompat.app.AppCompatActivity
import com.simongellis.vvb.databinding.ActivityGameBinding
import com.simongellis.vvb.emulator.Emulator
import com.simongellis.vvb.emulator.Renderer
import com.simongellis.vvb.emulator.Settings

class VideoPreviewActivity: AppCompatActivity() {
    private lateinit var _binding: ActivityGameBinding
    private lateinit var _emulator: Emulator
    private lateinit var _renderer: Renderer

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        _binding = ActivityGameBinding.inflate(layoutInflater)
        _emulator = Emulator.getInstance()
        _renderer = Renderer(_emulator, Settings(applicationContext))

        requestedOrientation = ActivityInfo.SCREEN_ORIENTATION_LANDSCAPE
        setContentView(_binding.root)

        val surfaceView = _binding.surfaceView
        surfaceView.setEGLContextClientVersion(2)
        surfaceView.setRenderer(_renderer)
        surfaceView.renderMode = GLSurfaceView.RENDERMODE_CONTINUOUSLY

        _emulator.loadImage(applicationContext)
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