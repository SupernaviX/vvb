package com.simongellis.vvb

import android.os.Bundle
import androidx.preference.PreferenceFragmentCompat

class AudioMenuFragment: PreferenceFragmentCompat() {
    override fun onCreatePreferences(savedInstanceState: Bundle?, rootKey: String?) {
        setPreferencesFromResource(R.xml.preferences_audio, rootKey)
    }
}