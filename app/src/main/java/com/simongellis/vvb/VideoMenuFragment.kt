package com.simongellis.vvb

import android.content.Intent
import android.os.Bundle
import androidx.preference.Preference
import androidx.preference.PreferenceFragmentCompat

class VideoMenuFragment: PreferenceFragmentCompat() {
    override fun onCreatePreferences(savedInstanceState: Bundle?, rootKey: String?) {
        addPreferencesFromResource(R.xml.preferences_video)

        findPreference<Preference>("video_preview")?.setOnPreferenceClickListener {
            preview()
            true
        }
    }

    private fun preview() {
        val intent = Intent(activity, VideoPreviewActivity::class.java)
        startActivity(intent)
    }
}