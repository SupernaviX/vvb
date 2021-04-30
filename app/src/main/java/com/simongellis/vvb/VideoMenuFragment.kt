package com.simongellis.vvb

import android.content.Intent
import android.content.SharedPreferences
import android.os.Bundle
import androidx.appcompat.app.AlertDialog
import androidx.preference.Preference
import androidx.preference.PreferenceFragmentCompat
import androidx.preference.PreferenceManager
import com.simongellis.vvb.emulator.VideoMode

class VideoMenuFragment: PreferenceFragmentCompat() {
    enum class Prefs(val prefName: String, val mode: VideoMode? = null) {
        MODE("video_mode"),
        ZOOM("video_screen_zoom_percent"),
        VERTICAL_OFFSET("video_vertical_offset"),
        COLOR("video_color", VideoMode.CARDBOARD),
        COLOR_LEFT("video_color_left", VideoMode.ANAGLYPH),
        COLOR_RIGHT("video_color_right", VideoMode.ANAGLYPH),
        SWITCH_VIEWER("video_switch_viewer", VideoMode.CARDBOARD),
        PREVIEW("video_preview"),
    }

    private lateinit var _sharedPreferences: SharedPreferences

    override fun onCreate(savedInstanceState: Bundle?) {
        _sharedPreferences = PreferenceManager.getDefaultSharedPreferences(context)
        super.onCreate(savedInstanceState)
    }

    override fun onCreatePreferences(savedInstanceState: Bundle?, rootKey: String?) {
        setPreferencesFromResource(R.xml.preferences_video, rootKey)

        val initialModeName = _sharedPreferences.getString(Prefs.MODE.prefName, VideoMode.ANAGLYPH.name)!!
        val initialMode = VideoMode.valueOf(initialModeName)
        hidePreferencesByMode(initialMode)

        findPref(Prefs.MODE).setOnPreferenceChangeListener { _, newValue ->
            val mode = VideoMode.valueOf(newValue as String)
            hidePreferencesByMode(mode)
            true
        }

        findPref(Prefs.COLOR_LEFT).setOnPreferenceChangeListener { _, newLeft ->
            val left = newLeft as Int
            val right = _sharedPreferences.getInt(Prefs.COLOR_RIGHT.prefName, 0xff0000ff.toInt())
            validateColors(left, right)
            true
        }

        findPref(Prefs.COLOR_RIGHT).setOnPreferenceChangeListener { _, newRight ->
            val left = _sharedPreferences.getInt(Prefs.COLOR_LEFT.prefName, 0xffff0000.toInt())
            val right = newRight as Int
            validateColors(left, right)
            true
        }

        findPref(Prefs.SWITCH_VIEWER).setOnPreferenceClickListener {
            val activity = activity as MainActivity
            activity.changeDeviceParams()
            true
        }

        findPref(Prefs.PREVIEW).setOnPreferenceClickListener {
            preview()
            true
        }
    }

    private fun validateColors(left: Int, right: Int) {
        if (doColorsOverlap(left, right)) {
            AlertDialog.Builder(requireContext())
                    .setTitle(R.string.video_menu_invalid_colors_title)
                    .setMessage(R.string.video_menu_invalid_colors_message)
                    .setPositiveButton(R.string.video_menu_invalid_colors_button, null)
                    .show()
        }
    }

    private fun doColorsOverlap(left: Int, right: Int): Boolean {
        for (byte in 0..2) {
            val leftByte = left.shr(byte * 8).and(0xff)
            val rightByte = right.shr(byte * 8).and(0xff)
            if (leftByte != 0 && rightByte != 0) {
                return true
            }
        }
        return false
    }

    private fun hidePreferencesByMode(mode: VideoMode) {
        Prefs.values().forEach { pref ->
            if (pref.mode != null) {
                findPref(pref).isVisible = pref.mode === mode
            }
        }
    }

    private fun findPref(pref: Prefs): Preference {
        return findPreference(pref.prefName)!!
    }

    private fun preview() {
        val intent = Intent(activity, VideoPreviewActivity::class.java)
        startActivity(intent)
    }
}