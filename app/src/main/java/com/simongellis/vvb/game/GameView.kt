package com.simongellis.vvb.game

import android.content.Context
import android.opengl.GLSurfaceView
import android.util.AttributeSet
import android.view.LayoutInflater
import android.widget.FrameLayout
import androidx.core.view.children
import androidx.core.view.isVisible
import com.simongellis.vvb.databinding.GameViewBinding
import com.simongellis.vvb.emulator.*

class GameView : FrameLayout {
    private val _binding: GameViewBinding
    private val _mode: VideoMode
    private val _renderer: Renderer

    var controller: Controller? = null
        set(value) {
            field = value
            for (control in controls) {
                control.controller = value
            }
        }

    private val controls
        get() = _binding.root.children.filterIsInstance<Control>()

    constructor(context: Context) : super(context)
    constructor(context: Context, attrs: AttributeSet?) : super(context, attrs)
    constructor(context: Context, attrs: AttributeSet?, defStyleAttr: Int) : super(context, attrs, defStyleAttr)
    @Suppress("unused")
    constructor(context: Context, attrs: AttributeSet?, defStyleAttr: Int, defStyleRes: Int) : super(context, attrs, defStyleAttr, defStyleRes)

    init {
        val emulator = Emulator.getInstance()
        val preferences = GamePreferences(context)
        _mode = preferences.videoMode
        _renderer = when(_mode) {
            VideoMode.ANAGLYPH -> AnaglyphRenderer(emulator, Settings(context))
            VideoMode.CARDBOARD -> CardboardRenderer(emulator, Settings(context))
        }

        val layoutInflater = LayoutInflater.from(context)
        _binding = GameViewBinding.inflate(layoutInflater, this, true)
        _binding.apply {
            surfaceView.setEGLContextClientVersion(2)
            surfaceView.setRenderer(_renderer)
            surfaceView.renderMode = GLSurfaceView.RENDERMODE_CONTINUOUSLY

            uiAlignmentMarker.isVisible = _mode === VideoMode.CARDBOARD
        }
        for (control in controls) {
            control.setPreferences(preferences)
        }
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