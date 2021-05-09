package com.simongellis.vvb.emulator

import android.content.Context
import android.graphics.Color
import androidx.preference.PreferenceManager

class Settings(context: Context) {
    // common video settings
    private val _screenZoom: Int
    private val _verticalOffset: Int

    // anaglyph video settings
    private val _colorLeft: Int
    private val _colorRight: Int

    // cardboard video settings
    private val _color: Int

    // audio settings
    private val _volume: Int
    private val _bufferSize: Int

    init {
        val prefs = PreferenceManager.getDefaultSharedPreferences(context)
        _screenZoom = prefs.getInt("video_screen_zoom_percent", 65)
        _verticalOffset = prefs.getInt("video_vertical_offset", 0)
        _colorLeft = prefs.getInt("video_color_left", Color.RED)
        _colorRight = prefs.getInt("video_color_right", Color.BLUE)
        _color = prefs.getInt("video_color", Color.RED)
        _volume = prefs.getInt("audio_volume", 100)
        _bufferSize = prefs.getInt("audio_buffer_size", 4)
    }
}