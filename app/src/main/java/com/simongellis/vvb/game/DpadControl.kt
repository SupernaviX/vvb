package com.simongellis.vvb.game

import android.content.Context
import android.graphics.*
import android.util.AttributeSet
import android.view.MotionEvent
import androidx.annotation.StyleableRes
import androidx.core.content.ContextCompat
import androidx.core.graphics.ColorUtils
import com.simongellis.vvb.R
import com.simongellis.vvb.emulator.Input

class DpadControl: Control {
    private val _centerGraphic = ContextCompat.getDrawable(context, R.drawable.ic_dpad_center)!!
    private val _arrowGraphic = ContextCompat.getDrawable(context, R.drawable.ic_dpad_arrow)!!
    private val _boundsPaint: Paint = Paint()

    private var _activeButtons = 0
    private lateinit var _inputs: Map<Arrow, Input>
    private val _rawPaths: Map<Arrow, Path>
    private var _paths: Map<Arrow, Path>

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

    init {
        _boundsPaint.color = ColorUtils.setAlphaComponent(Color.DKGRAY, 0x80)
        _rawPaths = Arrow.values()
            .mapIndexed { index, arrow -> arrow to computeArrowPath(index.toFloat() * 90f) }
            .toMap()
        _paths = _rawPaths
    }

    private fun computeArrowPath(rotation: Float): Path {
        val diagonalSensitivity = 1f/6f // 0f is none, .5f is max
        val deadZoneApothem = 1f/12f // half the width of the dead zone in the middle
        return Path().apply {
            moveTo(0f, diagonalSensitivity)
            lineTo(.5f - deadZoneApothem, .5f - deadZoneApothem)
            lineTo(.5f + deadZoneApothem, .5f - deadZoneApothem)
            lineTo(1f, diagonalSensitivity)
            lineTo(1f, 0f)
            lineTo(0f, 0f)
            close()
            transform(Matrix().apply {
                setRotate(rotation, .5f, .5f)
            })
        }
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

    override fun onSizeChanged(w: Int, h: Int, oldw: Int, oldh: Int) {
        val scaleMatrix = Matrix().apply {
            setScale(h.toFloat(), h.toFloat())
        }
        _paths = _rawPaths.mapValues { Path(it.value).apply { transform(scaleMatrix) } }
        super.onSizeChanged(w, h, oldw, oldh)
    }

    override fun drawGrayscale(canvas: Canvas, width: Int, height: Int) {
        if (shouldDrawBounds) {
            for (path in _paths.values) {
                canvas.drawPath(path, _boundsPaint)
            }
        }
        val mask = _activeButtons
        val pivotX = width.toFloat() / 2
        val pivotY = height.toFloat() / 2

        val visualLeft = width / 8
        val visualTop = height / 8
        val visualRight = width - visualLeft
        val visualBottom = height - visualTop

        _centerGraphic.setBounds(visualLeft, visualTop, visualRight, visualBottom)
        _centerGraphic.alpha = if (mask != 0) { 0xff } else { 0x80 }
        _centerGraphic.draw(canvas)

        _arrowGraphic.setBounds(visualLeft, visualTop, visualRight, visualBottom)
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

        val touch = Path().apply {
            addRect(event.x - 1, event.y - 1, event.x + 1, event.y + 1, Path.Direction.CW)
            transform(Matrix().apply {
                // adjust the touch to compensate for parallax
                setTranslate(-parallax / 2, 0f)
            })
        }

        var result = 0
        for ((arrow, path) in _paths) {
            val collision = Path(touch).apply { op(path, Path.Op.INTERSECT) }
            if (!collision.isEmpty) result += arrow.mask
        }

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