package com.simongellis.vvb.game

import android.content.Context
import android.content.pm.ActivityInfo
import android.graphics.Color
import android.opengl.GLSurfaceView
import android.util.AttributeSet
import android.view.LayoutInflater
import androidx.constraintlayout.widget.ConstraintLayout
import androidx.core.view.isVisible
import androidx.core.view.updateLayoutParams
import com.simongellis.vvb.databinding.GameViewBinding
import com.simongellis.vvb.emulator.*

class GameView : ConstraintLayout {
    private val _binding: GameViewBinding
    private val _renderer: Renderer

    var controller: Controller? = null
        set(value) {
            field = value
            _binding.gamepadView.controller = controller
        }

    val requestedOrientation: Int

    constructor(context: Context) : super(context)
    constructor(context: Context, attrs: AttributeSet?) : super(context, attrs)
    constructor(context: Context, attrs: AttributeSet?, defStyleAttr: Int) : super(context, attrs, defStyleAttr)
    @Suppress("unused")
    constructor(context: Context, attrs: AttributeSet?, defStyleAttr: Int, defStyleRes: Int) : super(context, attrs, defStyleAttr, defStyleRes)

    init {
        val emulator = Emulator.getInstance()
        val preferences = GamePreferences(context)
        _renderer = when(preferences.videoMode) {
            VideoMode.ANAGLYPH -> AnaglyphRenderer(emulator, preferences.anaglyphSettings)
            VideoMode.CARDBOARD -> CardboardRenderer(emulator, preferences.cardboardSettings)
        }

        val layoutInflater = LayoutInflater.from(context)
        _binding = GameViewBinding.inflate(layoutInflater, this)
        _binding.apply {
            startGuideline?.updateLayoutParams<LayoutParams> {
                guidePercent = preferences.horizontalOffset
            }

            surfaceView.setEGLContextClientVersion(2)
            surfaceView.setRenderer(_renderer)
            surfaceView.renderMode = GLSurfaceView.RENDERMODE_CONTINUOUSLY

            gamepadView.setPreferences(preferences)

            uiAlignmentMarker?.isVisible = preferences.videoMode === VideoMode.CARDBOARD
        }

        requestedOrientation = when(preferences.videoMode) {
            VideoMode.ANAGLYPH -> ActivityInfo.SCREEN_ORIENTATION_UNSPECIFIED
            VideoMode.CARDBOARD -> ActivityInfo.SCREEN_ORIENTATION_LANDSCAPE
        }

        setBackgroundColor(Color.BLACK)
    }

    fun onPause() {
        _binding.surfaceView.onPause()
    }

    fun onResume() {
        _binding.surfaceView.onResume()
        _renderer.onResume()
    }

    override fun onDetachedFromWindow() {
        super.onDetachedFromWindow()
        _renderer.destroy()
    }
}