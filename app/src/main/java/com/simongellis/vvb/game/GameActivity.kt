package com.simongellis.vvb.game

import android.os.Build
import androidx.appcompat.app.AppCompatActivity
import android.os.Bundle
import android.os.Handler
import android.view.KeyEvent
import android.view.MotionEvent
import android.view.View
import androidx.activity.viewModels
import com.simongellis.vvb.emulator.*

class GameActivity : AppCompatActivity() {
    private val viewModel: GameViewModel by viewModels()

    private lateinit var _view: GameView
    private lateinit var _audio: Audio
    private lateinit var _controller: Controller
    private lateinit var _inputBindingMapper: InputBindingMapper
    private lateinit var _preferences: GamePreferences

    private lateinit var mDecorView: View

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

        // Get a handle to the Window containing the UI.
        mDecorView = window.decorView
        // does this need re-bound onResume?
        mDecorView.setOnSystemUiVisibilityChangeListener(View.OnSystemUiVisibilityChangeListener { visibility: Int ->
            if (visibility and View.SYSTEM_UI_FLAG_FULLSCREEN == 0) {
                // Go back to immersive fullscreen mode in 3s
                val handler = Handler(mainLooper)
                handler.postDelayed(
                    { this.toggleImmersiveView(true) },
                    3000 /* 3s */
                )
            }
        })
        // should this move into the view?
        toggleImmersiveView(true)
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
        // should this move into the view?
        toggleImmersiveView(true)
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

    override fun onWindowFocusChanged(hasFocus: Boolean) {
        toggleImmersiveView(hasFocus)
    }

    // fires onCreate, onResume
    fun toggleImmersiveView(immersive: Boolean) {
        // todo: make this a user-configurable opt-in "optimization" (trade-off)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
            window.setSustainedPerformanceMode(immersive)
        }
        if(immersive && mDecorView !== null){
            mDecorView.systemUiVisibility = View.SYSTEM_UI_FLAG_IMMERSIVE_STICKY or
                    View.SYSTEM_UI_FLAG_LAYOUT_STABLE or
                    View.SYSTEM_UI_FLAG_LAYOUT_HIDE_NAVIGATION or
                    View.SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN or
                    View.SYSTEM_UI_FLAG_HIDE_NAVIGATION or
                    View.SYSTEM_UI_FLAG_FULLSCREEN or
                    View.SYSTEM_UI_FLAG_IMMERSIVE
        }
    }
}