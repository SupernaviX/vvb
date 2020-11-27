package com.simongellis.vvb

import android.content.pm.ActivityInfo
import android.opengl.GLSurfaceView
import androidx.appcompat.app.AppCompatActivity
import android.os.Bundle
import android.view.KeyEvent
import com.simongellis.vvb.databinding.ActivityGameBinding

class GameActivity : AppCompatActivity() {
    private lateinit var _binding: ActivityGameBinding
    private lateinit var _emulator: Emulator
    private lateinit var _renderer: Renderer
    private lateinit var _audio: Audio
    private lateinit var _inputBindingMapper: InputBindingMapper


    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        _binding = ActivityGameBinding.inflate(layoutInflater)
        _emulator = Emulator.getInstance(applicationContext)
        _renderer = Renderer(_emulator, Settings(applicationContext))
        _audio = Audio(_emulator)
        _inputBindingMapper = InputBindingMapper(applicationContext)

        requestedOrientation = ActivityInfo.SCREEN_ORIENTATION_LANDSCAPE
        setContentView(_binding.root)

        val surfaceView = _binding.surfaceView;
        surfaceView.setEGLContextClientVersion(2)
        surfaceView.setRenderer(_renderer)
        surfaceView.renderMode = GLSurfaceView.RENDERMODE_CONTINUOUSLY

        _emulator.loadImage()
    }

    override fun dispatchKeyEvent(event: KeyEvent): Boolean {
        val input = _inputBindingMapper.getBoundInput(event)
        if (input != null) {
            if (event.action == KeyEvent.ACTION_DOWN) {
                _emulator.keyDown(input)
            } else {
                _emulator.keyUp(input)
            }
            return true
        }
        return super.dispatchKeyEvent(event)
    }

    override fun onResume() {
        super.onResume()
        _binding.surfaceView.onResume()
        _renderer.ensureDeviceParams()
        // _emulator.loadImage()
        _audio.play()
        _emulator.resume()
    }

    override fun onPause() {
        super.onPause()
        _binding.surfaceView.onPause()
        _emulator.pause()
        _audio.pause()
    }

    override fun onDestroy() {
        super.onDestroy()
        _inputBindingMapper.destroy()
        // _audio.destroy()
        _renderer.destroy()
    }
}