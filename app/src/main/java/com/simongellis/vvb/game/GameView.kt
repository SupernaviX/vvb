package com.simongellis.vvb.game

import android.content.Context
import android.content.pm.ActivityInfo
import android.graphics.Color
import android.view.LayoutInflater
import androidx.constraintlayout.widget.ConstraintLayout
import androidx.core.view.isVisible
import androidx.core.view.updateLayoutParams
import com.simongellis.vvb.databinding.GameViewBinding
import com.simongellis.vvb.emulator.*

// Leia SDK Includes
import com.simongellis.vvb.leia.LeiaAdapter
import com.simongellis.vvb.leia.LeiaVersion

class GameView(context: Context) : ConstraintLayout(context), LeiaAdapter.BacklightListener {
    private val _binding: GameViewBinding
    private val _renderer: Renderer
    private val _preferences: GamePreferences

    // LitByLeia
    private val _leiaAdapter = LeiaAdapter.instance(context)

    var controller: Controller? = null
        set(value) {
            field = value
            _binding.gamepadView.controller = controller
        }

    val requestedOrientation: Int

    init {
        val emulator = Emulator.instance
        _preferences = GamePreferences(context)
        _renderer = when(_preferences.videoMode) {
            VideoMode.ANAGLYPH -> AnaglyphRenderer(emulator, _preferences.anaglyphSettings)
            VideoMode.CARDBOARD -> CardboardRenderer(emulator, _preferences.cardboardSettings)
            VideoMode.MONO_LEFT -> MonoRenderer(emulator, _preferences.monoSettings(Eye.LEFT))
            VideoMode.MONO_RIGHT -> MonoRenderer(emulator, _preferences.monoSettings(Eye.RIGHT))
            VideoMode.STEREO -> StereoRenderer(emulator, _preferences.stereoSettings)
            VideoMode.LEIA -> when(_leiaAdapter.leiaVersion) {
                LeiaVersion.Legacy -> LeiaRenderer(emulator, _preferences.leiaSettings)
                LeiaVersion.CNSDK -> CNSDKRenderer(emulator, _preferences.cnsdkSettings)
                null -> throw Exception("Device does not support leia")
            }
        }

        val layoutInflater = LayoutInflater.from(context)
        _binding = GameViewBinding.inflate(layoutInflater, this)
        _binding.apply {
            startGuideline?.updateLayoutParams<LayoutParams> {
                guidePercent = _preferences.horizontalOffset
            }

            surfaceView.setRenderer(_renderer)

            gamepadView.setPreferences(_preferences)

            uiAlignmentMarker?.isVisible = _preferences.videoMode === VideoMode.CARDBOARD
        }

        requestedOrientation = when(_preferences.videoMode.supportsPortrait) {
            true -> ActivityInfo.SCREEN_ORIENTATION_UNSPECIFIED
            false -> ActivityInfo.SCREEN_ORIENTATION_LANDSCAPE
        }

        setBackgroundColor(Color.BLACK)

        _leiaAdapter.registerBacklightListener(this@GameView)
        checkShouldToggle3D(_preferences.isLeia)
    }

    fun onPause() {
        _binding.surfaceView.onPause()
        checkShouldToggle3D(false)
    }

    fun onResume() {
        _binding.surfaceView.onResume()
        _renderer.onResume()
        checkShouldToggle3D(_preferences.isLeia)
    }

    override fun onDetachedFromWindow() {
        super.onDetachedFromWindow()
        _renderer.destroy()
    }

    override fun onWindowFocusChanged(hasWindowFocus: Boolean) {
        super.onWindowFocusChanged(hasWindowFocus)
        checkShouldToggle3D(_preferences.isLeia && hasWindowFocus)
    }

    override fun onBacklightChanged(enabled: Boolean) {
        if (_preferences.isLeia) {
            _renderer.onModeChanged(enabled)
        }
    }

    private fun checkShouldToggle3D(desiredState: Boolean) {
        if(_leiaAdapter.leiaVersion == null) {
            return
        }
        if (desiredState && _preferences.isLeia) {
            enable3D()
        } else {
            disable3D()
        }
    }

    private fun enable3D() {
        _leiaAdapter.enableBacklight()
    }

    private fun disable3D() {
        _leiaAdapter.disableBacklight()
    }
}