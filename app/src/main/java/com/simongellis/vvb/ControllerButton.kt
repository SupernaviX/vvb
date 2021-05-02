package com.simongellis.vvb

import android.content.Context
import android.graphics.*
import android.graphics.drawable.Drawable
import android.graphics.drawable.VectorDrawable
import android.util.AttributeSet
import androidx.annotation.ColorInt
import androidx.appcompat.widget.AppCompatImageButton
import kotlin.math.roundToInt

class ControllerButton : AppCompatImageButton {
    private lateinit var _rawDrawable: Drawable
    private var _regularWidth: Int = 0

    private var _parallax: Float = 0f
    private var _leftColor: Int = Color.RED
    private var _rightColor: Int = Color.BLUE

    constructor(context: Context): super(context) {
        init(context)
    }
    constructor(context: Context, attrs: AttributeSet?): super(context, attrs) {
        init(context, attrs)
    }
    constructor(context: Context, attrs: AttributeSet?, defStyleAttr: Int): super(context, attrs, defStyleAttr) {
        init(context, attrs, defStyleAttr)
    }

    private fun init(context: Context, attrs: AttributeSet? = null, defStyleAttr: Int = 0) {
        val a = context.theme.obtainStyledAttributes(attrs, R.styleable.ControllerButton, defStyleAttr, 0)

        try {
            _parallax = a.getDimension(R.styleable.ControllerButton_parallax, 0f)
        } finally {
            a.recycle()
        }
    }

    fun setColors(@ColorInt left: Int, @ColorInt right: Int) {
        _leftColor = left
        _rightColor = right
    }

    override fun setImageDrawable(drawable: Drawable?) {
        if (drawable is VectorDrawable) {
            // Only store the "raw" drawable if it's a VectorDrawable.
            // Assume a BitmapDrawable has already been parallax'd
            _rawDrawable = drawable
        }
        super.setImageDrawable(drawable)
    }

    override fun onMeasure(widthMeasureSpec: Int, heightMeasureSpec: Int) {
        super.onMeasure(widthMeasureSpec, heightMeasureSpec)
        _regularWidth = measuredWidth

        val widthWithParallax = measuredWidth + (_parallax * 2).roundToInt()
        setMeasuredDimension(widthWithParallax, measuredHeight)
    }

    override fun onSizeChanged(w: Int, h: Int, oldw: Int, oldh: Int) {
        super.onSizeChanged(w, h, oldw, oldh)
        setImageBitmap(buildParallaxBitmap(w, h))
    }

    private fun buildParallaxBitmap(width: Int, height: Int): Bitmap {
        val sourceBitmap = Bitmap.createBitmap(_regularWidth, height, Bitmap.Config.ARGB_8888)
        val sourceCanvas = Canvas(sourceBitmap)
        val targetBitmap = Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888)
        val targetCanvas = Canvas(targetBitmap)
        val paint = Paint()

        _rawDrawable.apply {
            setBounds(0, 0, _regularWidth, height)
            colorFilter = PorterDuffColorFilter(_leftColor, PorterDuff.Mode.MULTIPLY)
            draw(sourceCanvas)
        }

        targetCanvas.drawBitmap(sourceBitmap, 0f, 0f, paint)

        _rawDrawable.apply {
            colorFilter = PorterDuffColorFilter(_rightColor, PorterDuff.Mode.MULTIPLY)
            draw(sourceCanvas)
        }

        paint.xfermode = PorterDuffXfermode(PorterDuff.Mode.LIGHTEN)
        targetCanvas.drawBitmap(sourceBitmap, _parallax, 0f, paint)

        return targetBitmap
    }
}