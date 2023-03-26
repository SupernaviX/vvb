package com.simongellis.vvb.game

import android.content.Context
import android.content.pm.ActivityInfo
import android.graphics.Color
import android.util.AttributeSet
import android.view.LayoutInflater
import androidx.constraintlayout.widget.ConstraintLayout
import androidx.core.view.isVisible
import androidx.core.view.updateLayoutParams
import com.simongellis.vvb.databinding.GameViewBinding
import com.simongellis.vvb.emulator.*

// Leia SDK Includes
import com.leia.android.lights.LeiaDisplayManager
import com.leia.android.lights.LeiaDisplayManager.BacklightMode
import com.leia.android.lights.LeiaSDK
import com.leia.android.lights.BacklightModeListener
import com.leia.android.lights.LeiaDisplayManager.BacklightMode.MODE_2D
import com.leia.android.lights.LeiaDisplayManager.BacklightMode.MODE_3D

class GameView : ConstraintLayout, BacklightModeListener {
    private val _binding: GameViewBinding
    private val _renderer: Renderer
    private val _preferences: GamePreferences

    // LitByLeia
    private var mDisplayManager: LeiaDisplayManager? = null

    var controller: Controller? = null
        set(value) {
            field = value
            _binding.gamepadView.controller = controller
        }

    val requestedOrientation: Int

    private val _surfaceView: SurfaceViewAdapter
        get() = _binding.surfaceView as SurfaceViewAdapter

    constructor(context: Context) : super(context)
    constructor(context: Context, attrs: AttributeSet?) : super(context, attrs)
    constructor(context: Context, attrs: AttributeSet?, defStyleAttr: Int) : super(context, attrs, defStyleAttr)
    @Suppress("unused")
    constructor(context: Context, attrs: AttributeSet?, defStyleAttr: Int, defStyleRes: Int) : super(context, attrs, defStyleAttr, defStyleRes)

    init {
        val emulator = Emulator.instance
        _preferences = GamePreferences(context)
        _renderer = when(_preferences.videoMode) {
            VideoMode.ANAGLYPH -> AnaglyphRenderer(emulator, _preferences.anaglyphSettings)
            VideoMode.CARDBOARD -> CardboardRenderer(emulator, _preferences.cardboardSettings)
            VideoMode.MONO_LEFT -> MonoRenderer(emulator, _preferences.monoSettings(Eye.LEFT))
            VideoMode.MONO_RIGHT -> MonoRenderer(emulator, _preferences.monoSettings(Eye.RIGHT))
            VideoMode.STEREO -> StereoRenderer(emulator, _preferences.stereoSettings)
            VideoMode.LEIA -> LeiaRenderer(emulator, _preferences.leiaSettings)
        }

        val layoutInflater = LayoutInflater.from(context)
        _binding = GameViewBinding.inflate(layoutInflater, this)
        _binding.apply {
            startGuideline?.updateLayoutParams<LayoutParams> {
                guidePercent = _preferences.horizontalOffset
            }

            _surfaceView.setRenderer(_renderer)

            gamepadView.setPreferences(_preferences)

            uiAlignmentMarker?.isVisible = _preferences.videoMode === VideoMode.CARDBOARD
        }

        requestedOrientation = when(_preferences.videoMode.supportsPortrait) {
            true -> ActivityInfo.SCREEN_ORIENTATION_UNSPECIFIED
            false -> ActivityInfo.SCREEN_ORIENTATION_LANDSCAPE
        }

        setBackgroundColor(Color.BLACK)

        mDisplayManager = LeiaSDK.getDisplayManager(context)
        mDisplayManager?.apply {
            registerBacklightModeListener(this@GameView)
            checkShouldToggle3D(true)
        }
    }

    fun onPause() {
        _surfaceView.onPause()
        checkShouldToggle3D(false)
    }

    fun onResume() {
        _surfaceView.onResume()
        _renderer.onResume()
        checkShouldToggle3D(true)
    }

    override fun onDetachedFromWindow() {
        super.onDetachedFromWindow()
        _renderer.destroy()
    }

    override fun onWindowFocusChanged(hasWindowFocus: Boolean) {
        super.onWindowFocusChanged(hasWindowFocus)
        checkShouldToggle3D(_preferences.isLeia && hasWindowFocus)
    }

    /** BacklightModeListener Interface requirement  */
    override fun onBacklightModeChanged(backlightMode: BacklightMode) {
        if (_preferences.isLeia) {
            _renderer.onModeChanged(backlightMode == MODE_3D)
        }
    }

    private fun checkShouldToggle3D(desiredState: Boolean) {
        if(mDisplayManager === null) {
            return
        }
        if (desiredState && _preferences.isLeia) {
            enable3D()
        } else {
            disable3D()
        }
    }

    private fun enable3D() {
        mDisplayManager?.requestBacklightMode(MODE_3D)
    }

    private fun disable3D() {
        mDisplayManager?.requestBacklightMode(MODE_2D)
    }
}