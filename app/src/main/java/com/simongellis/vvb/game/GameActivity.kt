package com.simongellis.vvb.game

import androidx.appcompat.app.AppCompatActivity
import android.os.Bundle
import android.view.KeyEvent
import android.view.MotionEvent
import androidx.activity.viewModels
import com.simongellis.vvb.emulator.*

import com.leia.android.lights.LeiaDisplayManager
import com.leia.android.lights.LeiaDisplayManager.BacklightMode
import com.leia.android.lights.LeiaSDK
import com.leia.android.lights.BacklightModeListener
import com.leia.android.lights.LeiaDisplayManager.BacklightMode.MODE_2D
import com.leia.android.lights.LeiaDisplayManager.BacklightMode.MODE_3D

class GameActivity : AppCompatActivity(), BacklightModeListener {
    private val viewModel: GameViewModel by viewModels()

    private lateinit var _view: GameView
    private lateinit var _audio: Audio
    private lateinit var _controller: Controller
    private lateinit var _inputBindingMapper: InputBindingMapper
    private lateinit var _preferences: GamePreferences

    // LitByLeia
    private var mRenderModeIsLeia3d = false
    private var prev_desired_backlight_state = false
    private val mExpectedBacklightMode: BacklightMode? = null
    private var mBacklightHasShutDown = false
    private var mIsDeviceCurrentlyInPortraitMode = false
    private var mDisplayManager: LeiaDisplayManager? = null

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

        // TODO if device is LitByLeia
        mDisplayManager = LeiaSDK.getDisplayManager(this)
        mDisplayManager?.registerBacklightModeListener(this)
        checkShouldToggle3D(true)

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
        checkShouldToggle3D(true)
    }

    override fun onPause() {
        super.onPause()
        viewModel.pauseGame()
        _audio.stop()
        _view.onPause()
        checkShouldToggle3D(false)
    }

    override fun onDestroy() {
        super.onDestroy()
        _inputBindingMapper.destroy()
        _view.controller = null
        _controller.destroy()
        _audio.destroy()
        checkShouldToggle3D(false)
    }

    /** BacklightModeListener Interface requirement  */
    override fun onBacklightModeChanged(backlightMode: BacklightMode) {
        //Log.e("EmulationActivity", "onBacklightModeChanged: callback received");
        // Do something to remember the backlight is no longer on
        // Later, we have to let the native side know this occurred.
        if (mExpectedBacklightMode == MODE_3D &&
            mExpectedBacklightMode != backlightMode
        ) {
            //Log.e("EmulationActivity", "onBacklightModeChanged: mBacklightHasShutDown = true;");
            mBacklightHasShutDown = true
        }
    }


    fun checkShouldToggle3D(desired_state: Boolean) {
        if (desired_state && _preferences.isLeia) {
            Enable3D()
        } else {
            Disable3D()
        }
        prev_desired_backlight_state = desired_state
    }

    fun Enable3D() {
        mDisplayManager?.setBacklightMode(MODE_3D)
    }

    fun Disable3D() {
        mDisplayManager?.setBacklightMode(MODE_2D)
    }
}