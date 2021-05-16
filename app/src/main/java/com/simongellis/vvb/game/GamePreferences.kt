package com.simongellis.vvb.game

import android.content.Context
import android.content.res.Configuration
import android.graphics.Color
import android.util.DisplayMetrics
import androidx.annotation.ColorInt
import androidx.preference.PreferenceManager
import com.simongellis.vvb.emulator.AnaglyphRenderer
import com.simongellis.vvb.emulator.Audio
import com.simongellis.vvb.emulator.CardboardRenderer
import com.simongellis.vvb.emulator.VideoMode

class GamePreferences(context: Context) {
    val videoMode: VideoMode
    private val isAnaglyph
        get() = videoMode == VideoMode.ANAGLYPH

    private val isPortrait
        = context.resources.configuration.orientation == Configuration.ORIENTATION_PORTRAIT

    private val _screenZoom: Int
    private val screenZoom
        get() = if (isPortrait && isAnaglyph) { 100 } else { _screenZoom }
    private val _verticalOffset: Int
    private val verticalOffset
        get() = if (isPortrait && isAnaglyph) { 0 } else { _verticalOffset }

    @ColorInt val colorLeft: Int
    @ColorInt val colorRight: Int

    @ColorInt val color: Int

    private val _virtualGamepadOn: Boolean
    val showVirtualGamepad
        get() = isAnaglyph && _virtualGamepadOn
    val toggleMode: Boolean
    val enableHapticFeedback: Boolean
    val controlParallax: Float
    val showControlBounds: Boolean

    private val volume: Int
    private val bufferSize: Int

    val audioSettings
        get() = Audio.Settings(volume, bufferSize)

    val anaglyphSettings
        get() = AnaglyphRenderer.Settings(screenZoom, verticalOffset, colorLeft, colorRight)

    val cardboardSettings
        get() = CardboardRenderer.Settings(screenZoom, verticalOffset, color)

    init {

        val prefs = PreferenceManager.getDefaultSharedPreferences(context)

        videoMode = VideoMode.valueOf(prefs.getString("video_mode", VideoMode.ANAGLYPH.name)!!)

        _screenZoom = prefs.getInt("video_screen_zoom_percent", 65)
        _verticalOffset = prefs.getInt("video_vertical_offset", 0)

        colorLeft = prefs.getInt("video_color_left", Color.RED)
        colorRight = prefs.getInt("video_color_right", Color.BLUE)

        color = prefs.getInt("video_color", Color.RED)

        volume = prefs.getInt("audio_volume", 100)
        bufferSize = prefs.getInt("audio_buffer_size", 4)

        _virtualGamepadOn = prefs.getBoolean("onscreen_input_on", true)
        toggleMode = prefs.getBoolean("onscreen_input_toggle_controls", false)
        enableHapticFeedback = prefs.getBoolean("onscreen_input_haptic_feedback", true)
        controlParallax = convertDpToPixels(context,
            prefs.getInt("onscreen_input_parallax", 8).toFloat())
        showControlBounds = prefs.getBoolean("onscreen_input_show_bounds", false)
    }

    private fun convertDpToPixels(context: Context, dp: Float): Float {
        return dp * context.resources.displayMetrics.densityDpi / DisplayMetrics.DENSITY_DEFAULT
    }
}