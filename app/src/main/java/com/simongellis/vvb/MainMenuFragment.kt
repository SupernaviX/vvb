package com.simongellis.vvb

import android.app.Activity
import android.content.Intent
import android.os.Bundle
import android.view.View
import androidx.preference.Preference
import androidx.preference.PreferenceFragmentCompat

class MainMenuFragment: PreferenceFragmentCompat() {
    private val GAME_CHOSEN = 2

    override fun onCreatePreferences(savedInstanceState: Bundle?, rootKey: String?) {
        addPreferencesFromResource(R.xml.preferences)

        findPreference<Preference>("resume_game")?.setOnPreferenceClickListener {
            playGame()
            true
        }

        findPreference<Preference>("load_game")?.setOnPreferenceClickListener { preference ->
            startActivityForResult(preference.intent, GAME_CHOSEN)
            true
        }

        findPreference<Preference>("switch_viewer")?.setOnPreferenceClickListener { _ ->
            val activity = activity as MainActivity
            activity.changeDeviceParams()
            true
        }
    }

    override fun onResume() {
        super.onResume()
        activity!!.setTitle(R.string.app_name)
        findPreference<Preference>("resume_game")?.apply {
            val emulator = Emulator.getInstance(context!!)
            isVisible = emulator.isGameLoaded()
        }
    }

    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)
        if (requestCode == GAME_CHOSEN && resultCode == Activity.RESULT_OK) {
            data?.data?.also { uri ->
                val emulator = Emulator.getInstance(context!!)
                emulator.loadGamePak(uri)
                playGame()
            }
        }
    }

    private fun playGame() {
        val intent = Intent(activity, GameActivity::class.java)
        startActivity(intent)
    }
}