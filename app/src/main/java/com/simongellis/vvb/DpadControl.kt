package com.simongellis.vvb

import android.content.Context
import android.graphics.Canvas
import android.util.AttributeSet
import android.view.MotionEvent
import androidx.core.content.ContextCompat

class DpadControl: Control {
    private val _centerGraphic = ContextCompat.getDrawable(context, R.drawable.ic_dpad_center)!!
    private val _arrowGraphic = ContextCompat.getDrawable(context, R.drawable.ic_dpad_arrow)!!

    private var _activeButtons = 0

    constructor(context: Context) : super(context)
    constructor(context: Context, attrs: AttributeSet?) : super(context, attrs)
    constructor(context: Context, attrs: AttributeSet?, defStyleAttr: Int) : super(context, attrs, defStyleAttr)
    @Suppress("unused")
    constructor(context: Context, attrs: AttributeSet?, defStyleAttr: Int, defStyleRes: Int) : super(context, attrs, defStyleAttr, defStyleRes)

    init {
        setOnTouchListener { v, event ->
            _activeButtons = getPressed(event)
            drawingState = _activeButtons
            v.performClick()
            true
        }
    }

    override fun drawGrayscale(canvas: Canvas) {
        val mask = _activeButtons
        val pivotX = width.toFloat() / 2
        val pivotY = height.toFloat() / 2

        _centerGraphic.setBounds(0, 0, width, height)
        _centerGraphic.alpha = if (mask != 0) { 0xff } else { 0x80 }
        _centerGraphic.draw(canvas)

        _arrowGraphic.setBounds(0, 0, width, height)
        for (arrow in Arrow.values()) {
            _arrowGraphic.alpha = if (arrow.isPressed(mask)) { 0xff } else { 0x80 }
            _arrowGraphic.draw(canvas)
            canvas.rotate(90f, pivotX, pivotY)
        }
    }

    private fun getPressed(event: MotionEvent): Int {
        if (event.action == MotionEvent.ACTION_UP) {
            return 0
        }
        val xRegion = event.x / width
        val yRegion = event.y / height

        var result = 0
        if (xRegion < 0.35f) result += Arrow.LEFT.mask
        if (xRegion > 0.65f) result += Arrow.RIGHT.mask
        if (yRegion < 0.35f) result += Arrow.UP.mask
        if (yRegion > 0.65f) result += Arrow.DOWN.mask
        return result
    }

    enum class Arrow(val mask: Int) {
        UP(0x01),
        RIGHT(0x02),
        DOWN(0x04),
        LEFT(0x08);

        fun isPressed(mask: Int): Boolean {
            return this.mask.and(mask) != 0
        }
    }
}