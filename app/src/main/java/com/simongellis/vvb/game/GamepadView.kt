package com.simongellis.vvb.game

import android.content.Context
import android.util.AttributeSet
import android.view.LayoutInflater
import androidx.constraintlayout.widget.ConstraintLayout
import androidx.core.view.children
import com.simongellis.vvb.databinding.GamepadViewBinding
import com.simongellis.vvb.emulator.Controller

class GamepadView : ConstraintLayout {
    private val controls
        get() = children.filterIsInstance<Control>()

    constructor(context: Context) : super(context)
    constructor(context: Context, attrs: AttributeSet?) : super(context, attrs)
    constructor(context: Context, attrs: AttributeSet?, defStyleAttr: Int) : super(context, attrs, defStyleAttr)
    @Suppress("unused")
    constructor(context: Context, attrs: AttributeSet?, defStyleAttr: Int, defStyleRes: Int) : super(context, attrs, defStyleAttr, defStyleRes)

    init {
        val inflater = LayoutInflater.from(context)
        GamepadViewBinding.inflate(inflater, this)
    }

    fun setPreferences(preferences: GamePreferences) {
        visibility = if (preferences.showVirtualGamepad) { VISIBLE } else { INVISIBLE }
        for (control in controls) {
            control.setPreferences(preferences)
        }
    }

    var controller: Controller? = null
        set(value) {
            field = value
            for (control in controls) {
                control.controller = value
            }
        }

}