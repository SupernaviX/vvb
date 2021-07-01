package com.simongellis.vvb.menu

import android.content.Context
import android.content.res.TypedArray
import android.util.AttributeSet
import androidx.preference.DialogPreference
import com.simongellis.vvb.game.VideoMode

class VideoModeDialogPreference(context: Context, attrs: AttributeSet): DialogPreference(context, attrs) {
    private var selected: VideoMode? = null

    override fun onGetDefaultValue(a: TypedArray, index: Int): VideoMode? {
        val raw = a.getString(index)
        return raw?.let { VideoMode.valueOf(it) }
    }

    override fun onSetInitialValue(defaultValue: Any?) {
        val raw = getPersistedString(null)
        val parsed = raw?.let { VideoMode.valueOf(it) }
        setValue(parsed ?: (defaultValue as VideoMode?))
    }

    override fun getSummary(): String? {
        val context = context ?: return null
        return selected?.let { context.getString(it.summary) }
    }

    fun getValue(): VideoMode? {
        return selected
    }
    fun setValue(value: VideoMode?) {
        selected = value
        val raw = value?.toString() ?: return
        if (callChangeListener(raw) && persistString(raw)) {
            notifyChanged()
        }
    }
}