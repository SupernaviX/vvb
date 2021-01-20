package com.simongellis.vvb

import android.app.Activity
import android.content.Intent
import android.os.Bundle
import androidx.preference.Preference
import androidx.preference.PreferenceFragmentCompat

class MainMenuFragment: PreferenceFragmentCompat() {
    private val GAME_CHOSEN = 2

    override fun onCreatePreferences(savedInstanceState: Bundle?, rootKey: String?) {
        setPreferencesFromResource(R.xml.preferences, rootKey)

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
        requireActivity().setTitle(R.string.app_name)
        findPreference<Preference>("resume_game")?.apply {
            val emulator = Emulator.getInstance()
            isVisible = emulator.isGameLoaded()
        }
    }

    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)
        if (requestCode == GAME_CHOSEN && resultCode == Activity.RESULT_OK) {
            data?.data?.also { uri ->
                val emulator = Emulator.getInstance()
                emulator.loadGamePak(requireContext(), uri)
                playGame()
            }
        }
    }

    private fun playGame() {
        val intent = Intent(activity, GameActivity::class.java)
        startActivity(intent)
    }
}