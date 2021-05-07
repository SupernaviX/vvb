package com.simongellis.vvb

import android.content.Context
import android.graphics.Canvas
import android.util.AttributeSet
import android.view.MotionEvent.ACTION_UP
import androidx.core.content.ContextCompat
import com.simongellis.vvb.emulator.Input

class ButtonControl: Control {
    private val _button = ContextCompat.getDrawable(context, R.drawable.ic_button)!!

    private var _isPressed = false
    private var _input: Input? = null

    constructor(context: Context) : super(context) {
        init(context, null)
    }
    constructor(context: Context, attrs: AttributeSet?) : super(context, attrs) {
        init(context, attrs)
    }
    constructor(context: Context, attrs: AttributeSet?, defStyleAttr: Int) : super(context, attrs, defStyleAttr) {
        init(context, attrs)
    }
    @Suppress("unused")
    constructor(context: Context, attrs: AttributeSet?, defStyleAttr: Int, defStyleRes: Int) : super(context, attrs, defStyleAttr, defStyleRes) {
        init(context, attrs)
    }

    private fun init(context: Context, attrs: AttributeSet?) {
        val a = context.obtainStyledAttributes(attrs, R.styleable.ButtonControl)

        try {
            val inputStr = a.getString(R.styleable.ButtonControl_input)
            _input = inputStr?.let { Input.valueOf(it) }
        } finally {
            a.recycle()
        }

        setOnTouchListener { v, event ->
            val wasPressed = _isPressed
            _isPressed = event.action != ACTION_UP

            if (_isPressed && !wasPressed) {
                _input?.also { controller?.press(it) }
            }
            if (wasPressed && !_isPressed) {
                _input?.also { controller?.release(it) }
            }
            drawingState = if (_isPressed) { 1 } else { 0 }

            v.performClick()
            true
        }
    }

    override fun drawGrayscale(canvas: Canvas, width: Int, height: Int) {
        _button.setBounds(0, 0, width, height)
        _button.alpha = if (_isPressed) { 0xff } else { 0x80 }
        _button.draw(canvas)
    }

}