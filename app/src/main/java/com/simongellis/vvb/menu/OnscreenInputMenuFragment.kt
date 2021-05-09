package com.simongellis.vvb.menu

import android.content.Intent
import android.os.Bundle
import androidx.preference.Preference
import androidx.preference.PreferenceFragmentCompat
import com.simongellis.vvb.R
import com.simongellis.vvb.game.PreviewActivity

class OnscreenInputMenuFragment: PreferenceFragmentCompat() {
    override fun onCreatePreferences(savedInstanceState: Bundle?, rootKey: String?) {
        setPreferencesFromResource(R.xml.preferences_onscreen_input, rootKey)

        findPreference<Preference>("onscreen_input_preview")!!.setOnPreferenceClickListener {
            preview()
            true
        }
    }

    private fun preview() {
        val intent = Intent(activity, PreviewActivity::class.java)
        startActivity(intent)
    }
}