package com.simongellis.vvb

import android.content.Context
import android.graphics.Canvas
import android.util.AttributeSet
import android.view.MotionEvent
import android.view.MotionEvent.ACTION_DOWN
import android.view.MotionEvent.ACTION_UP
import androidx.core.content.ContextCompat
import com.simongellis.vvb.emulator.Input

class ButtonControl: Control {
    private val _button = ContextCompat.getDrawable(context, R.drawable.ic_button)!!

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
        val isToggle: Boolean

        try {
            val inputStr = a.getString(R.styleable.ButtonControl_input)
            _input = inputStr?.let { Input.valueOf(it) }
            isToggle = a.getBoolean(R.styleable.ButtonControl_toggleable, false)
        } finally {
            a.recycle()
        }

        val onTouch = if (isToggle) { handleToggle() } else { ::handleTouch }

        setOnTouchListener { v, event ->
            onTouch(event)
            v.performClick()
            true
        }
    }

    private fun handleTouch(event: MotionEvent) {
        isPressed = event.action != ACTION_UP
    }

    private fun handleToggle(): (MotionEvent) -> Unit {
        var isToggled = false
        return { event ->
            if (event.action == ACTION_DOWN) {
                isPressed = true
            }
            if (event.action == ACTION_UP) {
                if (isToggled) {
                    isPressed = false
                }
                isToggled = !isToggled
            }
        }
    }

    override fun setPressed(pressed: Boolean) {
        if (isPressed == pressed) return
        super.setPressed(pressed)
        if (pressed) {
            _input?.also { controller?.press(it) }
        } else {
            _input?.also { controller?.release(it) }
        }
        drawingState = if (pressed) { 1 } else { 0 }
    }

    override fun drawGrayscale(canvas: Canvas, width: Int, height: Int) {
        _button.setBounds(0, 0, width, height)
        _button.alpha = if (isPressed) { 0xff } else { 0x80 }
        _button.draw(canvas)
    }

}