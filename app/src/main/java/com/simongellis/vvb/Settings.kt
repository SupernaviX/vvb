package com.simongellis.vvb

import android.content.Context
import androidx.preference.PreferenceManager

class Settings(context: Context) {
    private val _screenZoom: Int
    private val _verticalOffset: Int

    init {
        val prefs = PreferenceManager.getDefaultSharedPreferences(context)
        _screenZoom = prefs.getInt("video_screen_zoom_percent", 65)
        _verticalOffset = prefs.getInt("video_vertical_offset", 0)
    }
}