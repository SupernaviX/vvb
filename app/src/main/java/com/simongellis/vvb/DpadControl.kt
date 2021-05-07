package com.simongellis.vvb

import android.content.Context
import android.graphics.Canvas
import android.util.AttributeSet
import android.view.MotionEvent
import androidx.annotation.StyleableRes
import androidx.core.content.ContextCompat
import com.simongellis.vvb.emulator.Input

class DpadControl: Control {
    private val _centerGraphic = ContextCompat.getDrawable(context, R.drawable.ic_dpad_center)!!
    private val _arrowGraphic = ContextCompat.getDrawable(context, R.drawable.ic_dpad_arrow)!!

    private var _activeButtons = 0
    private lateinit var _inputs: Map<Arrow, Input>

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
        val a = context.obtainStyledAttributes(attrs, R.styleable.DpadControl)

        try {
            _inputs = Arrow.values()
                .map { arrow -> arrow to a.getString(arrow.id) }
                .mapNotNull { p -> p.second?.let { p.first to Input.valueOf(it) } }
                .toMap()
        } finally {
            a.recycle()
        }

        setOnTouchListener { v, event ->
            val oldActiveButtons = _activeButtons
            val newActiveButtons = getPressed(event)
            _activeButtons = newActiveButtons

            val justPressed = newActiveButtons.and(oldActiveButtons.inv())
            val justReleased = oldActiveButtons.and(newActiveButtons.inv())
            for (arrow in Arrow.values()) {
                if (arrow.isIn(justPressed)) {
                    _inputs[arrow]?.also { controller?.press(it) }
                }
                if (arrow.isIn(justReleased)) {
                    _inputs[arrow]?.also { controller?.release(it) }
                }
            }

            drawingState = _activeButtons
            v.performClick()
            true
        }
    }

    override fun drawGrayscale(canvas: Canvas, width: Int, height: Int) {
        val mask = _activeButtons
        val pivotX = width.toFloat() / 2
        val pivotY = height.toFloat() / 2

        _centerGraphic.setBounds(0, 0, width, height)
        _centerGraphic.alpha = if (mask != 0) { 0xff } else { 0x80 }
        _centerGraphic.draw(canvas)

        _arrowGraphic.setBounds(0, 0, width, height)
        for (arrow in Arrow.values()) {
            _arrowGraphic.alpha = if (arrow.isIn(mask)) { 0xff } else { 0x80 }
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

    enum class Arrow(val mask: Int, @StyleableRes val id: Int) {
        UP(0x01, R.styleable.DpadControl_inputUp),
        RIGHT(0x02, R.styleable.DpadControl_inputRight),
        DOWN(0x04, R.styleable.DpadControl_inputDown),
        LEFT(0x08, R.styleable.DpadControl_inputLeft);

        fun isIn(mask: Int): Boolean {
            return this.mask.and(mask) != 0
        }
    }
}