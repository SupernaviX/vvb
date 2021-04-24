package com.simongellis.vvb

import android.content.Intent
import android.content.SharedPreferences
import android.os.Bundle
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