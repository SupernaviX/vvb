package com.simongellis.vvb

import android.content.Context
import android.graphics.*
import android.graphics.drawable.BitmapDrawable
import android.graphics.drawable.Drawable
import android.util.AttributeSet
import android.view.View
import androidx.annotation.ColorInt
import com.simongellis.vvb.emulator.Controller
import kotlin.math.roundToInt

abstract class Control: View {
    private var _parallax: Float = 0f
    private var _leftColor: Int = Color.RED
    private var _rightColor: Int = Color.BLUE
    private var _drawable: Drawable? = null
    var controller: Controller? = null

    constructor(context: Context) : super(context) {
        init(context)
    }
    constructor(context: Context, attrs: AttributeSet?) : super(context, attrs) {
        init(context, attrs)
    }
    constructor(context: Context, attrs: AttributeSet?, defStyleAttr: Int) : super(context, attrs, defStyleAttr) {
        init(context, attrs, defStyleAttr)
    }
    @Suppress("unused")
    constructor(context: Context, attrs: AttributeSet?, defStyleAttr: Int, defStyleRes: Int) : super(context, attrs, defStyleAttr, defStyleRes) {
        init(context, attrs, defStyleAttr, defStyleRes)
    }

    private fun init(context: Context, attrs: AttributeSet? = null, defStyleAttr: Int = 0, defStyleRes: Int = 0) {
        val a = context.theme.obtainStyledAttributes(attrs, R.styleable.Control, defStyleAttr, defStyleRes)

        try {
            _parallax = a.getDimension(R.styleable.Control_parallax, 0f)
        } finally {
            a.recycle()
        }
    }

    fun setColors(@ColorInt left: Int, @ColorInt right: Int) {
        _leftColor = left
        _rightColor = right
    }

    override fun onMeasure(widthMeasureSpec: Int, heightMeasureSpec: Int) {
        super.onMeasure(widthMeasureSpec, heightMeasureSpec)
        setMeasuredDimension(measuredWidth + _parallax.roundToInt(), measuredHeight)
    }

    override fun onSizeChanged(w: Int, h: Int, oldw: Int, oldh: Int) {
        super.onSizeChanged(w, h, oldw, oldh)
        recomputeDrawable()
    }

    /**
     * Basic mechanism to only recompute the grayscale effect when needed
     */
    protected var drawingState = 0
        set(value) {
            if (field == value) return
            field = value
            recomputeDrawable()
        }

    /**
     * Draw a grayscale version of this control's graphic.
     */
    abstract fun drawGrayscale(canvas: Canvas, width: Int, height: Int)

    override fun onDraw(canvas: Canvas) {
        _drawable?.apply {
            setBounds(0, 0, width, height)
            draw(canvas)
        }
    }

    private fun recomputeDrawable() {
        val naturalWidth = width - _parallax.roundToInt()
        val source = Bitmap.createBitmap(naturalWidth, height, Bitmap.Config.ARGB_8888)
        drawGrayscale(Canvas(source), naturalWidth, height)

        val bitmap = Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888)
        val canvas = Canvas(bitmap)

        val paint = Paint()
        paint.colorFilter = PorterDuffColorFilter(_leftColor, PorterDuff.Mode.MULTIPLY)
        canvas.drawBitmap(source, 0f, 0f, paint)

        paint.colorFilter = PorterDuffColorFilter(_rightColor, PorterDuff.Mode.MULTIPLY)
        paint.xfermode = PorterDuffXfermode(PorterDuff.Mode.LIGHTEN)
        canvas.drawBitmap(source, _parallax, 0f, paint)

        _drawable = BitmapDrawable(resources, bitmap)
        invalidate()
    }
}