package com.simongellis.vvb.game

import android.content.Context
import android.content.SharedPreferences
import android.content.res.Configuration
import android.graphics.Color
import android.util.DisplayMetrics
import androidx.annotation.ColorInt
import androidx.core.content.ContextCompat
import androidx.preference.PreferenceManager
import com.simongellis.vvb.emulator.*
import com.simongellis.vvb.R
import com.simongellis.vvb.leia.LeiaAdapter

class GamePreferences(context: Context) {
    val videoMode: VideoMode
    private val isAnaglyph
        get() = videoMode == VideoMode.ANAGLYPH

    val isLeia
        get() = videoMode == VideoMode.LEIA

    private val supportsPortrait
        get() = videoMode.supportsPortrait
    private val isPortrait
        = context.resources.configuration.orientation == Configuration.ORIENTATION_PORTRAIT

    private val screenZoom: Float
        get() = if (isPortrait && supportsPortrait) { 1.00f } else { field }
    private val aspectRatio: AspectRatio
        get() = if (isPortrait && supportsPortrait) { AspectRatio.AUTO } else { field }

    // Horizontal offset is handled by the GameView, so that everything on screen is shifted
    val horizontalOffset: Float
        get() = if (isPortrait && supportsPortrait) { 0f } else { field }
    // Vertical offset is handled by the Renderer implementations in Rust,
    // because it specifically affects _the image rendered by_ Google Cardboard
    private val verticalOffset: Float
        get() = if (isPortrait && supportsPortrait) { 0f } else { field }

    @ColorInt val color: Int

    @ColorInt val colorBG: Int

    @ColorInt val colorLeft: Int
        get() = if (isAnaglyph) { field } else { color }
    @ColorInt val colorRight: Int
        get() = if (isAnaglyph) { field } else { color }

    private val _virtualGamepadOn: Boolean
    val showVirtualGamepad
        get() = supportsPortrait && _virtualGamepadOn
    val toggleMode: Boolean
    val enableHapticFeedback: Boolean
    val controlParallax: Float
        get() = if (isAnaglyph) { field } else { 0f }
    val showControlBounds: Boolean

    private val volume: Float
    private val bufferSize: Int

    val audioSettings
        get() = Audio.Settings(volume, bufferSize)

    val anaglyphSettings
        get() = AnaglyphRenderer.Settings(screenZoom, aspectRatio.ordinal, verticalOffset, colorLeft, colorRight)

    val cardboardSettings
        get() = CardboardRenderer.Settings(screenZoom, aspectRatio.ordinal, verticalOffset, color)

    fun monoSettings(eye: Eye)
        = MonoRenderer.Settings(eye.ordinal, screenZoom, aspectRatio.ordinal, verticalOffset, color)

    val stereoSettings
        get() = StereoRenderer.Settings(screenZoom, aspectRatio.ordinal, verticalOffset, color)

    val cnsdkSettings
        get() = CNSDKRenderer.Settings(screenZoom, aspectRatio.ordinal, verticalOffset, color)

    val leiaSettings
        get() = LeiaRenderer.Settings(screenZoom, aspectRatio.ordinal, verticalOffset, color, colorBG)

    val sustainedPerformanceModeOn: Boolean

    init {

        val prefs = PreferenceManager.getDefaultSharedPreferences(context)
        val supportsLeia = LeiaAdapter.instance(context).leiaVersion != null

        var defaultMode = VideoMode.ANAGLYPH.name
        var defaultScreenZoom = 100

        if (supportsLeia) {
            defaultMode = VideoMode.LEIA.name
            defaultScreenZoom = 65
        }

        videoMode = VideoMode.valueOf(prefs.getString("video_mode", defaultMode)!!)

        screenZoom = prefs.getIntPercent("video_screen_zoom_percent", defaultScreenZoom)
        aspectRatio = AspectRatio.valueOf(prefs.getString("video_aspect_ratio", "AUTO")!!)
        horizontalOffset = prefs.getIntPercent("video_horizontal_offset", 0)
        verticalOffset = prefs.getIntPercent("video_vertical_offset", 0)

        colorLeft = prefs.getInt("video_color_left", Color.RED)
        colorRight = prefs.getInt("video_color_right", Color.BLUE)

        var defaultBGColor = Color.BLACK
        if (supportsLeia) {
            defaultBGColor = ContextCompat.getColor(context, R.color.leia_grey)
        }
        colorBG = prefs.getInt("video_color_bg", defaultBGColor)

        color = prefs.getInt("video_color", Color.RED)

        volume = prefs.getIntPercent("audio_volume", 100)
        bufferSize = prefs.getInt("audio_buffer_size", 4)

        _virtualGamepadOn = prefs.getBoolean("onscreen_input_on", true)
        toggleMode = prefs.getBoolean("onscreen_input_toggle_controls", false)
        enableHapticFeedback = prefs.getBoolean("onscreen_input_haptic_feedback", true)
        controlParallax = convertDpToPixels(context,
            prefs.getInt("onscreen_input_parallax", 8).toFloat())
        showControlBounds = prefs.getBoolean("onscreen_input_show_bounds", false)

        sustainedPerformanceModeOn = prefs.getBoolean("sustained_performance_mode_on", false)
    }

    private fun convertDpToPixels(context: Context, dp: Float): Float {
        return dp * context.resources.displayMetrics.densityDpi / DisplayMetrics.DENSITY_DEFAULT
    }

    private fun SharedPreferences.getIntPercent(key: String, defValue: Int): Float {
        val percent = getInt(key, defValue)
        return percent.toFloat() / 100f
    }
}
