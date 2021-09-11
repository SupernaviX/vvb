package com.simongellis.vvb.menu

import android.app.Dialog
import android.content.Intent
import android.content.SharedPreferences
import android.graphics.Color
import android.os.Bundle
import androidx.appcompat.app.AlertDialog
import androidx.preference.ListPreference
import androidx.preference.Preference
import androidx.preference.PreferenceFragmentCompat
import androidx.preference.PreferenceManager
import com.simongellis.vvb.R
import com.simongellis.vvb.emulator.VvbLibrary
import com.simongellis.vvb.game.PreviewActivity
import com.simongellis.vvb.game.VideoMode

class VideoMenuFragment: PreferenceFragmentCompat() {
    enum class Prefs(val prefName: String, vararg val modes: VideoMode = VideoMode.values()) {
        MODE("video_mode"),
        ASPECT_RATIO("video_aspect_ratio"),
        ZOOM("video_screen_zoom_percent"),
        HORIZONTAL_OFFSET("video_horizontal_offset"),
        VERTICAL_OFFSET("video_vertical_offset"),
        COLOR("video_color", VideoMode.CARDBOARD, VideoMode.MONO_LEFT, VideoMode.MONO_RIGHT, VideoMode.STEREO),
        COLOR_LEFT("video_color_left", VideoMode.ANAGLYPH),
        COLOR_RIGHT("video_color_right", VideoMode.ANAGLYPH),
        SWITCH_VIEWER("video_switch_viewer", VideoMode.CARDBOARD),
        PREVIEW("video_preview"),
    }

    private lateinit var _sharedPreferences: SharedPreferences
    private var _dialog: Dialog? = null
        set(value) {
            field = value
            value?.setOnDismissListener { field = null }
        }

    override fun onCreate(savedInstanceState: Bundle?) {
        _sharedPreferences = PreferenceManager.getDefaultSharedPreferences(context)
        super.onCreate(savedInstanceState)
    }

    override fun onPause() {
        super.onPause()
        _dialog?.apply { dismiss() }
    }

    override fun onResume() {
        super.onResume()
        requireActivity().setTitle(R.string.main_menu_video_setup)
    }

    override fun onCreatePreferences(savedInstanceState: Bundle?, rootKey: String?) {
        setPreferencesFromResource(R.xml.preferences_video, rootKey)

        val initialModeName = _sharedPreferences.getString(Prefs.MODE.prefName, VideoMode.ANAGLYPH.name)!!
        val initialMode = VideoMode.valueOf(initialModeName)
        hidePreferencesByMode(initialMode)

        val videoModePref = findPreference<DetailedListPreference>(Prefs.MODE.prefName)
        videoModePref?.detailedEntries = VideoMode.values().map {
            val summary = getString(it.summary)
            val description = getString(it.description)
            DetailedListPreference.Entry(it.name, summary, description)
        }

        findPref(Prefs.MODE).setOnPreferenceChangeListener { _, newValue ->
            val mode = VideoMode.valueOf(newValue as String)
            hidePreferencesByMode(mode)
            true
        }

        findPref(Prefs.ASPECT_RATIO).setSummaryProvider {
            val pref = it as ListPreference
            val value = pref.value ?: pref.entryValues.first()
            val index = pref.entryValues.indexOf(value)
            pref.entries[index]
        }

        findPref(Prefs.COLOR_LEFT).setOnPreferenceChangeListener { _, newLeft ->
            val left = newLeft as Int
            val right = _sharedPreferences.getInt(Prefs.COLOR_RIGHT.prefName, Color.BLUE)
            validateColors(left, right)
            true
        }

        findPref(Prefs.COLOR_RIGHT).setOnPreferenceChangeListener { _, newRight ->
            val left = _sharedPreferences.getInt(Prefs.COLOR_LEFT.prefName, Color.RED)
            val right = newRight as Int
            validateColors(left, right)
            true
        }

        findPref(Prefs.SWITCH_VIEWER).setOnPreferenceClickListener {
            VvbLibrary.instance.changeDeviceParams()
            true
        }

        findPref(Prefs.PREVIEW).setOnPreferenceClickListener {
            preview()
            true
        }
    }

    private fun validateColors(left: Int, right: Int) {
        if (doColorsOverlap(left, right)) {
            _dialog = AlertDialog.Builder(requireContext())
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
            findPref(pref).isVisible = pref.modes.contains(mode)
        }
    }

    private fun findPref(pref: Prefs): Preference {
        return findPreference(pref.prefName)!!
    }

    private fun preview() {
        val intent = Intent(activity, PreviewActivity::class.java)
        startActivity(intent)
    }
}