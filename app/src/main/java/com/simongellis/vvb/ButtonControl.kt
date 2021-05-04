package com.simongellis.vvb

import android.content.Context
import android.graphics.Canvas
import android.util.AttributeSet
import android.view.MotionEvent.ACTION_DOWN
import androidx.core.content.ContextCompat

class ButtonControl: Control {
    private val _button = ContextCompat.getDrawable(context, R.drawable.ic_button)!!

    private var _isPressed = false

    constructor(context: Context) : super(context)
    constructor(context: Context, attrs: AttributeSet?) : super(context, attrs)
    constructor(context: Context, attrs: AttributeSet?, defStyleAttr: Int) : super(context, attrs, defStyleAttr)
    @Suppress("unused")
    constructor(context: Context, attrs: AttributeSet?, defStyleAttr: Int, defStyleRes: Int) : super(context, attrs, defStyleAttr, defStyleRes)

    init {
        setOnTouchListener { v, event ->
            _isPressed = event.action == ACTION_DOWN
            drawingState = if (_isPressed) { 1 } else { 0 }
            v.performClick()
            true
        }
    }

    override fun drawGrayscale(canvas: Canvas) {
        _button.setBounds(0, 0, width, height)
        _button.alpha = if (_isPressed) { 0xff } else { 0x80 }
        _button.draw(canvas)
    }

}