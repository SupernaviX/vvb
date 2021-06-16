package com.simongellis.vvb.menu

import android.content.Context
import android.util.AttributeSet
import androidx.preference.Preference
import androidx.preference.PreferenceViewHolder

class ControllerInputPreference: Preference {
    private var _onLongClickListener: (() -> Boolean)? = null

    @Suppress("unused")
    constructor(context: Context) : super(context)
    @Suppress("unused")
    constructor(context: Context, attrs: AttributeSet?) : super(context, attrs)
    @Suppress("unused")
    constructor(context: Context, attrs: AttributeSet?, defStyleAttr: Int) : super(context, attrs, defStyleAttr)

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