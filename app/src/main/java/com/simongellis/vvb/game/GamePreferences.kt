package com.simongellis.vvb.game

import android.content.Context
import android.graphics.Color
import android.util.DisplayMetrics
import androidx.annotation.ColorInt
import androidx.preference.PreferenceManager
import com.simongellis.vvb.emulator.VideoMode

class GamePreferences(context: Context) {
    val videoMode: VideoMode
    @ColorInt val colorLeft: Int
    @ColorInt val colorRight: Int

    private val _virtualGamepadOn: Boolean
    val showVirtualGamepad
        get() = videoMode == VideoMode.ANAGLYPH && _virtualGamepadOn
    val toggleMode: Boolean
    val enableHapticFeedback: Boolean
    val controlParallax: Float
    val showControlBounds: Boolean

    init {
        val prefs = PreferenceManager.getDefaultSharedPreferences(context)

        videoMode = VideoMode.valueOf(prefs.getString("video_mode", VideoMode.ANAGLYPH.name)!!)
        colorLeft = prefs.getInt("video_color_left", Color.RED)
        colorRight = prefs.getInt("video_color_right", Color.BLUE)

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