package com.simongellis.vvb.menu

import android.content.Intent
import android.os.Bundle
import androidx.activity.result.contract.ActivityResultContracts.OpenDocument
import androidx.preference.Preference
import androidx.preference.PreferenceFragmentCompat
import com.simongellis.vvb.R
import com.simongellis.vvb.emulator.Emulator
import com.simongellis.vvb.game.GameActivity

class MainMenuFragment: PreferenceFragmentCompat() {
    override fun onCreatePreferences(savedInstanceState: Bundle?, rootKey: String?) {
        setPreferencesFromResource(R.xml.preferences, rootKey)

        findPreference<Preference>("resume_game")?.setOnPreferenceClickListener {
            playGame()
            true
        }

        val chooseGame = registerForActivityResult(OpenDocument()) { uri ->
            val emulator = Emulator.getInstance()
            emulator.loadGamePak(requireContext(), uri)
            playGame()
        }
        findPreference<Preference>("load_game")?.setOnPreferenceClickListener {
            chooseGame.launch(arrayOf("*/*"))
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

    private fun playGame() {
        val intent = Intent(activity, GameActivity::class.java)
        startActivity(intent)
    }
}