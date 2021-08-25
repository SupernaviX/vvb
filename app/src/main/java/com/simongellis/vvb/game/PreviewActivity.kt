package com.simongellis.vvb.game

import android.os.Bundle
import androidx.activity.viewModels
import androidx.appcompat.app.AppCompatActivity
import com.simongellis.vvb.emulator.Emulator
import com.simongellis.vvb.emulator.VvbLibrary

import com.leia.android.lights.LeiaDisplayManager
import com.leia.android.lights.LeiaDisplayManager.BacklightMode
import com.leia.android.lights.LeiaSDK
import com.leia.android.lights.BacklightModeListener
import com.leia.android.lights.LeiaDisplayManager.BacklightMode.MODE_2D
import com.leia.android.lights.LeiaDisplayManager.BacklightMode.MODE_3D

class PreviewActivity: AppCompatActivity(), BacklightModeListener {
    private val viewModel: GameViewModel by viewModels()

    private lateinit var _view: GameView
    private lateinit var _preferences: GamePreferences

    // LitByLeia
    private var mRenderModeIsLeia3d = false
    private var prev_desired_backlight_state = false
    private val mExpectedBacklightMode: LeiaDisplayManager.BacklightMode? = null
    private var mBacklightHasShutDown = false
    private var mIsDeviceCurrentlyInPortraitMode = false
    private var mDisplayManager: LeiaDisplayManager? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        VvbLibrary.instance.initialize(this)

        _view = GameView(baseContext)
        _preferences = GamePreferences(baseContext)
        requestedOrientation = _view.requestedOrientation
        setContentView(_view)

        viewModel.loadPreviewImage()

        // TODO if device is LitByLeia
        mDisplayManager = LeiaSDK.getDisplayManager(this)
        mDisplayManager?.registerBacklightModeListener(this)
        checkShouldToggle3D(true)
    }

    override fun onPause() {
        super.onPause()
        _view.onPause()
        checkShouldToggle3D(false)
    }

    override fun onResume() {
        super.onResume()
        _view.onResume()
        viewModel.loadPreviewImage()
        checkShouldToggle3D(true)
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
        mDisplayManager?.setBacklightMode(LeiaDisplayManager.BacklightMode.MODE_3D)
    }

    fun Disable3D() {
        mDisplayManager?.setBacklightMode(LeiaDisplayManager.BacklightMode.MODE_2D)
    }
}