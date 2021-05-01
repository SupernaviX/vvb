package com.simongellis.vvb

import android.content.Context
import android.opengl.GLSurfaceView
import android.util.AttributeSet
import android.view.LayoutInflater
import androidx.constraintlayout.widget.ConstraintLayout
import androidx.core.view.isVisible
import com.simongellis.vvb.databinding.GameViewBinding
import com.simongellis.vvb.emulator.*

class GameView : ConstraintLayout {
    private val _binding: GameViewBinding
    private val _mode: VideoMode
    private val _renderer: Renderer

    constructor(context: Context) : super(context)
    constructor(context: Context, attrs: AttributeSet?) : super(context, attrs)
    constructor(context: Context, attrs: AttributeSet?, defStyleAttr: Int) : super(context, attrs, defStyleAttr)
    @Suppress("unused")
    constructor(context: Context, attrs: AttributeSet?, defStyleAttr: Int, defStyleRes: Int) : super(context, attrs, defStyleAttr, defStyleRes)

    init {
        val emulator = Emulator.getInstance()
        val settings = Settings(context)
        _mode = settings.videoMode
        _renderer = when(_mode) {
            VideoMode.ANAGLYPH -> AnaglyphRenderer(emulator, settings)
            VideoMode.CARDBOARD -> CardboardRenderer(emulator, settings)
        }

        val layoutInflater = LayoutInflater.from(context)
        _binding = GameViewBinding.inflate(layoutInflater, this, true)
        _binding.apply {
            surfaceView.setEGLContextClientVersion(2)
            surfaceView.setRenderer(_renderer)
            surfaceView.renderMode = GLSurfaceView.RENDERMODE_CONTINUOUSLY

            uiAlignmentMarker.isVisible = _mode === VideoMode.CARDBOARD
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