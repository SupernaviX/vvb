package com.simongellis.vvb.menu

import android.os.Bundle
import androidx.preference.PreferenceFragmentCompat
import com.simongellis.vvb.R

class GeneralMenuFragment: PreferenceFragmentCompat() {
    override fun onCreatePreferences(savedInstanceState: Bundle?, rootKey: String?) {
        setPreferencesFromResource(R.xml.preferences_general, rootKey)
    }

    override fun onResume() {
        super.onResume()
        requireActivity().setTitle(R.string.main_menu_general_setup)
    }
}