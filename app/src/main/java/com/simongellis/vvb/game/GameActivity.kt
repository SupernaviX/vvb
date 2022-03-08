package com.simongellis.vvb.game

import android.content.res.Configuration
import androidx.appcompat.app.AppCompatActivity
import android.os.Bundle
import android.view.KeyEvent
import android.view.MotionEvent
import androidx.activity.viewModels
import androidx.core.view.ViewCompat
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import com.simongellis.vvb.emulator.*

class GameActivity : AppCompatActivity() {
    private val viewModel: GameViewModel by viewModels()

    private lateinit var _view: GameView
    private lateinit var _audio: Audio
    private lateinit var _controller: Controller

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        VvbLibrary.instance.initialize(this)

        val emulator = Emulator.instance
        val preferences = GamePreferences(baseContext)

        _audio = Audio(emulator, preferences.audioSettings)
        _controller = Controller(emulator)

        _view = GameView(baseContext)
        requestedOrientation = _view.requestedOrientation
        _view.controller = _controller
        setContentView(_view)

        if (resources.configuration.orientation == Configuration.ORIENTATION_LANDSCAPE) {
            hideSystemUI()
        } else {
            showSystemUI()
        }

        viewModel.loadPreviewImage()
    }

    private fun hideSystemUI() {
        WindowCompat.setDecorFitsSystemWindows(window, false)
        ViewCompat.getWindowInsetsController(window.decorView)?.apply {
            systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
            hide(WindowInsetsCompat.Type.systemBars())
        }
    }

    private fun showSystemUI() {
        WindowCompat.setDecorFitsSystemWindows(window, true)
        ViewCompat.getWindowInsetsController(window.decorView)?.apply {
            show(WindowInsetsCompat.Type.systemBars())
        }
    }

    override fun dispatchKeyEvent(event: KeyEvent): Boolean {
        val input = viewModel.getBoundInput(event)
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
        val (pressed, released) = viewModel.getAxisInputs(event)
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
        _view.controller = null
        _controller.destroy()
        _audio.destroy()
    }
}