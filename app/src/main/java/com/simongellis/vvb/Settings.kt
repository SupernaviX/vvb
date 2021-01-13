package com.simongellis.vvb

import android.content.Context
import androidx.preference.PreferenceManager

class Settings(context: Context) {
    private val _screenZoom: Int
    private val _verticalOffset: Int
    private val _color: Int
    private val _volume: Int
    private val _bufferSize: Int

    init {
        val prefs = PreferenceManager.getDefaultSharedPreferences(context)
        _screenZoom = prefs.getInt("video_screen_zoom_percent", 65)
        _verticalOffset = prefs.getInt("video_vertical_offset", 0)
        _color = prefs.getInt("video_color", 0xff0000ff.toInt())
        _volume = prefs.getInt("audio_volume", 100)
        _bufferSize = prefs.getInt("audio_buffer_size", 4)
    }
}