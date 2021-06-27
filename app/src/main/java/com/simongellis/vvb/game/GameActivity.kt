package com.simongellis.vvb.game

import androidx.appcompat.app.AppCompatActivity
import android.os.Bundle
import android.view.KeyEvent
import android.view.MotionEvent
import com.simongellis.vvb.emulator.*

class GameActivity : AppCompatActivity() {
    private lateinit var _view: GameView
    private lateinit var _emulator: Emulator
    private lateinit var _audio: Audio
    private lateinit var _controller: Controller
    private lateinit var _inputBindingMapper: InputBindingMapper

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        VvbLibrary.instance.initialize(this)

        _emulator = Emulator.instance
        val preferences = GamePreferences(baseContext)

        _audio = Audio(_emulator, preferences.audioSettings)
        _controller = Controller(_emulator)
        _inputBindingMapper = InputBindingMapper(baseContext)

        _view = GameView(baseContext)
        requestedOrientation = _view.requestedOrientation
        _view.controller = _controller
        setContentView(_view)

        _emulator.loadImage(baseContext)
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

    override fun dispatchGenericMotionEvent(event: MotionEvent): Boolean {
        val (pressed, released) = _inputBindingMapper.getAxisInputs(event)
        if (pressed.isNotEmpty() || released.isNotEmpty()) {
            _controller.update(pressed, released)
            return true
        }
        return super.dispatchGenericMotionEvent(event)
    }

    override fun onResume() {
        super.onResume()
        _view.onResume()
        _audio.start()
        _emulator.resume()
    }

    override fun onPause() {
        super.onPause()
        _view.onPause()
        _emulator.pause()
        _audio.stop()
    }

    override fun onDestroy() {
        super.onDestroy()
        _inputBindingMapper.destroy()
        _controller.destroy()
        _audio.destroy()
    }
}