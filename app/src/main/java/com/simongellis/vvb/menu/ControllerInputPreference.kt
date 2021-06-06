package com.simongellis.vvb.menu

import android.content.Context
import android.util.AttributeSet
import androidx.preference.Preference
import androidx.preference.PreferenceViewHolder
import com.simongellis.vvb.R
import com.simongellis.vvb.emulator.Input
import com.simongellis.vvb.game.ControllerDao

class ControllerInputPreference: Preference {
    private val _input: Input
    private var _onLongClickListener: (() -> Boolean)? = null

    @Suppress("unused")
    constructor(context: Context) : super(context) {
        _input = init(context, null)
    }
    @Suppress("unused")
    constructor(context: Context, attrs: AttributeSet?) : super(context, attrs) {
        _input = init(context, attrs)
    }
    @Suppress("unused")
    constructor(context: Context, attrs: AttributeSet?, defStyleAttr: Int) : super(context, attrs, defStyleAttr) {
        _input = init(context, attrs)
    }

    private fun init(context: Context, attrs: AttributeSet?): Input {
        val a = context.obtainStyledAttributes(attrs, R.styleable.ControllerInputPreference)

        try {
            val inputStr = a.getString(R.styleable.ControllerInputPreference_input)
            return Input.valueOf(inputStr!!)
        } finally {
            a.recycle()
        }
    }

    fun setMappings(mappings: List<ControllerDao.Mapping>) {
        if (mappings.isEmpty()) {
            setSummary(R.string.input_menu_unmapped)
        } else {
            summary = mappings.joinToString(", ")
        }
    }

    fun setIsBinding(multiple: Boolean) {
        if (multiple) {
            setSummary(R.string.input_menu_add_mapping)
        } else {
            setSummary(R.string.input_menu_put_mapping)
        }
    }

    override fun onBindViewHolder(holder: PreferenceViewHolder) {
        super.onBindViewHolder(holder)
        val view = holder.itemView
        view.setOnLongClickListener {
            _onLongClickListener?.invoke() ?: false
        }
    }

    fun setOnClickListener(listener: () -> Boolean) {
        this.setOnPreferenceClickListener {
            listener()
        }
    }

    fun setOnLongClickListener(listener: () -> Boolean) {
        _onLongClickListener = listener
    }
}