package com.simongellis.vvb.game

import androidx.appcompat.app.AppCompatActivity
import android.os.Bundle
import android.view.KeyEvent
import android.view.MotionEvent
import androidx.activity.viewModels
import com.simongellis.vvb.emulator.*

class GameActivity : AppCompatActivity() {
    private val viewModel: GameViewModel by viewModels()

    private lateinit var _view: GameView
    private lateinit var _audio: Audio
    private lateinit var _controller: Controller
    private lateinit var _inputBindingMapper: InputBindingMapper
    private lateinit var _preferences: GamePreferences

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        VvbLibrary.instance.initialize(this)

        val emulator = Emulator.instance
        val preferences = GamePreferences(baseContext)

        _audio = Audio(emulator, preferences.audioSettings)
        _controller = Controller(emulator)
        _inputBindingMapper = InputBindingMapper(baseContext)

        _view = GameView(baseContext)
        requestedOrientation = _view.requestedOrientation
        _view.controller = _controller
        setContentView(_view)
        _preferences = preferences

        viewModel.loadPreviewImage()
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
        viewModel.resumeGame()
    }

    override fun onPause() {
        super.onPause()
        viewModel.pauseGame()
        _audio.stop()
        _view.onPause()
    }

    override fun onDestroy() {
        super.onDestroy()
        _inputBindingMapper.destroy()
        _view.controller = null
        _controller.destroy()
        _audio.destroy()
    }
}