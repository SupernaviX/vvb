package com.simongellis.vvb

import android.content.pm.ActivityInfo
import androidx.appcompat.app.AppCompatActivity
import android.os.Bundle
import android.view.KeyEvent
import com.simongellis.vvb.emulator.*

class GameActivity : AppCompatActivity() {
    private lateinit var _view: GameView
    private lateinit var _emulator: Emulator
    private lateinit var _audio: Audio
    private lateinit var _controller: Controller
    private lateinit var _inputBindingMapper: InputBindingMapper

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        _emulator = Emulator.getInstance()
        val settings = Settings(applicationContext)

        _audio = Audio(_emulator, settings)
        _controller = Controller(_emulator)
        _inputBindingMapper = InputBindingMapper(applicationContext)

        requestedOrientation = ActivityInfo.SCREEN_ORIENTATION_LANDSCAPE
        _view = GameView(applicationContext)
        _view.controller = _controller
        setContentView(_view)

        _emulator.loadImage(applicationContext)
    }

    override fun dispatchKeyEvent(event: KeyEvent): Boolean {
        val input = _inputBindingMapper.getBoundInput(event)
        if (input != null) {
            if (event.action == KeyEvent.ACTION_DOWN) {
                _controller.press(input)
            } else {
                _controller.release(input)
            }
            return true
        }
        return super.dispatchKeyEvent(event)
    }

    override fun onResume() {
        super.onResume()
        _view.onResume()
        _audio.play()
        _emulator.resume()
    }

    override fun onPause() {
        super.onPause()
        _view.onPause()
        _emulator.pause()
        _audio.pause()
    }

    override fun onDestroy() {
        super.onDestroy()
        _inputBindingMapper.destroy()
        _controller.destroy()
        _audio.destroy()
    }
}