package com.simongellis.vvb.game

import android.content.Context
import android.graphics.*
import android.graphics.drawable.BitmapDrawable
import android.graphics.drawable.Drawable
import android.os.Build
import android.util.AttributeSet
import android.view.HapticFeedbackConstants
import android.view.View
import androidx.core.view.isVisible
import com.simongellis.vvb.emulator.Controller
import kotlin.math.roundToInt

abstract class Control: View {
    private var _leftColor: Int = Color.RED
    private var _rightColor: Int = Color.BLUE
    private var _drawable: Drawable? = null

    protected var parallax: Float = 0f
    protected var shouldDrawBounds = false

    var controller: Controller? = null

    constructor(context: Context) : super(context)
    constructor(context: Context, attrs: AttributeSet?) : super(context, attrs)
    constructor(context: Context, attrs: AttributeSet?, defStyleAttr: Int) : super(context, attrs, defStyleAttr)
    @Suppress("unused")
    constructor(context: Context, attrs: AttributeSet?, defStyleAttr: Int, defStyleRes: Int) : super(context, attrs, defStyleAttr, defStyleRes)

    open fun setPreferences(preferences: GamePreferences) {
        isVisible = preferences.showVirtualGamepad
        _leftColor = preferences.colorLeft
        _rightColor = preferences.colorRight
        isHapticFeedbackEnabled = preferences.enableHapticFeedback
        parallax = preferences.controlParallax
        shouldDrawBounds = preferences.showControlBounds
    }

    override fun onMeasure(widthMeasureSpec: Int, heightMeasureSpec: Int) {
        super.onMeasure(widthMeasureSpec, heightMeasureSpec)
        setMeasuredDimension(measuredWidth + parallax.roundToInt(), measuredHeight)
    }

    override fun onSizeChanged(w: Int, h: Int, oldw: Int, oldh: Int) {
        super.onSizeChanged(w, h, oldw, oldh)
        recomputeDrawable()
    }

    fun performHapticPress() {
        performHapticFeedback(HapticFeedbackConstants.VIRTUAL_KEY)
    }

    fun performHapticRelease() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O_MR1) {
            performHapticFeedback(HapticFeedbackConstants.VIRTUAL_KEY_RELEASE)
        }
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
        val naturalWidth = width - parallax.roundToInt()
        val source = Bitmap.createBitmap(naturalWidth, height, Bitmap.Config.ARGB_8888)
        drawGrayscale(Canvas(source), naturalWidth, height)

        val bitmap = Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888)
        val canvas = Canvas(bitmap)

        val paint = Paint()
        paint.colorFilter = PorterDuffColorFilter(_leftColor, PorterDuff.Mode.MULTIPLY)
        canvas.drawBitmap(source, 0f, 0f, paint)

        paint.colorFilter = PorterDuffColorFilter(_rightColor, PorterDuff.Mode.MULTIPLY)
        paint.xfermode = PorterDuffXfermode(PorterDuff.Mode.LIGHTEN)
        canvas.drawBitmap(source, parallax, 0f, paint)

        _drawable = BitmapDrawable(resources, bitmap)
        invalidate()
    }
}