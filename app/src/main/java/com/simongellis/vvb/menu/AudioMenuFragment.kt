package com.simongellis.vvb.menu

import android.os.Bundle
import androidx.preference.PreferenceFragmentCompat
import com.simongellis.vvb.R

class AudioMenuFragment: PreferenceFragmentCompat() {
    override fun onCreatePreferences(savedInstanceState: Bundle?, rootKey: String?) {
        setPreferencesFromResource(R.xml.preferences_audio, rootKey)
    }

    override fun onResume() {
        super.onResume()
        requireActivity().setTitle(R.string.main_menu_audio_setup)
    }
}