package com.simongellis.vvb.emulator

import android.content.Context
import android.graphics.Color
import androidx.preference.PreferenceManager

class Settings(context: Context) {
    // common video settings
    val videoMode: VideoMode
    private val _screenZoom: Int
    private val _verticalOffset: Int

    // anaglyph video settings
    private val _colorLeft: Int
    private val _colorRight: Int

    val colorLeft: Int get() = _colorLeft
    val colorRight: Int get() = _colorRight

    // cardboard video settings
    private val _color: Int

    // audio settings
    private val _volume: Int
    private val _bufferSize: Int

    init {
        val prefs = PreferenceManager.getDefaultSharedPreferences(context)
        videoMode = VideoMode.valueOf(prefs.getString("video_mode", VideoMode.ANAGLYPH.name)!!)
        _screenZoom = prefs.getInt("video_screen_zoom_percent", 65)
        _verticalOffset = prefs.getInt("video_vertical_offset", 0)
        _colorLeft = prefs.getInt("video_color_left", Color.RED)
        _colorRight = prefs.getInt("video_color_right", Color.BLUE)
        _color = prefs.getInt("video_color", Color.RED)
        _volume = prefs.getInt("audio_volume", 100)
        _bufferSize = prefs.getInt("audio_buffer_size", 4)
    }
}