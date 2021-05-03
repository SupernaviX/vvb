package com.simongellis.vvb

import android.content.Context
import android.util.AttributeSet
import android.view.LayoutInflater
import android.view.MotionEvent
import androidx.appcompat.widget.AppCompatImageView
import androidx.constraintlayout.widget.ConstraintLayout
import com.simongellis.vvb.databinding.DpadBinding

class ControllerDpad: ConstraintLayout {
    private val _binding: DpadBinding
    private var _activeButtons: Int = 0

    constructor(context: Context) : super(context)
    constructor(context: Context, attrs: AttributeSet?) : super(context, attrs)
    constructor(context: Context, attrs: AttributeSet?, defStyleAttr: Int) : super(context, attrs, defStyleAttr)
    @Suppress("unused")
    constructor(context: Context, attrs: AttributeSet?, defStyleAttr: Int, defStyleRes: Int) : super(context, attrs, defStyleAttr, defStyleRes)

    init {
        val layoutInflater = LayoutInflater.from(context)
        _binding = DpadBinding.inflate(layoutInflater, this, true)
        _binding.center.alpha = 0.5f
        _binding.up.alpha = 0.5f
        _binding.down.alpha = 0.5f
        _binding.left.alpha = 0.5f
        _binding.right.alpha = 0.5f

        setOnTouchListener { v, event ->
            val newActiveButtons = getPressed(event)
            val oldActiveButtons = _activeButtons
            _activeButtons = newActiveButtons

            val justPressed = newActiveButtons.and(oldActiveButtons.inv())
            val justReleased = oldActiveButtons.and(newActiveButtons.inv())
            val anyJustPressed = newActiveButtons != 0 && oldActiveButtons == 0
            val allJustReleased = oldActiveButtons != 0 && newActiveButtons == 0

            for (pressed in getValues(justPressed)) {
                pressed.getView(_binding).alpha = 1f
            }
            for (released in getValues(justReleased)) {
                released.getView(_binding).alpha = 0.5f
            }
            if (anyJustPressed) {
                _binding.center.alpha = 1f
            }
            if (allJustReleased) {
                _binding.center.alpha = 0.5f
            }

            v.performClick()
            true
        }
    }

    private fun getPressed(event: MotionEvent): Int {
        if (event.action == MotionEvent.ACTION_UP) {
            return 0
        }
        val xRegion = event.x / width
        val yRegion = event.y / height

        var result = 0
        if (xRegion < 1f/3) result += DpadPress.LEFT.mask
        if (xRegion > 2f/3) result += DpadPress.RIGHT.mask
        if (yRegion < 1f/3) result += DpadPress.UP.mask
        if (yRegion > 2f/3) result += DpadPress.DOWN.mask
        return result
    }

    private fun getValues(mask: Int): List<DpadPress> {
        return DpadPress.values().toList().filter { v -> v.mask.and(mask) != 0 }
    }

    enum class DpadPress(val mask: Int, val getView: (binding: DpadBinding) -> AppCompatImageView) {
        LEFT(0x01, { it.left }),
        RIGHT(0x02, { it.right }),
        UP(0x04, { it.up }),
        DOWN(0x08, { it.down });
    }

}