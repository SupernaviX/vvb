package com.simongellis.vvb

import android.os.Bundle
import androidx.preference.PreferenceFragmentCompat

class VideoMenuFragment: PreferenceFragmentCompat() {
    override fun onCreatePreferences(savedInstanceState: Bundle?, rootKey: String?) {
        addPreferencesFromResource(R.xml.preferences_video)
    }
}