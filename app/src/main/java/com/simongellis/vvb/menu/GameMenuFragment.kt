package com.simongellis.vvb.menu

import android.content.Intent
import android.os.Bundle
import androidx.fragment.app.viewModels
import androidx.preference.Preference
import androidx.preference.PreferenceFragmentCompat
import com.simongellis.vvb.MainActivity
import com.simongellis.vvb.MainViewModel
import com.simongellis.vvb.R
import com.simongellis.vvb.game.GameActivity
import com.simongellis.vvb.utils.observeNow

class GameMenuFragment: PreferenceFragmentCompat() {
    private val viewModel: MainViewModel by viewModels({ requireActivity() })
    private var title: String = ""

    override fun onCreatePreferences(savedInstanceState: Bundle?, rootKey: String?) {
        setPreferencesFromResource(R.xml.preferences_game, rootKey)

        findPreference<Preference>("resume_game")?.setOnPreferenceClickListener {
            viewModel.openGame()
            playGame()
            true
        }
        findPreference<Preference>("reset_game")?.setOnPreferenceClickListener {
            viewModel.resetGame()
            playGame()
            true
        }
        findPreference<Preference>("save_state")?.setOnPreferenceClickListener {
            viewModel.saveState()
            true
        }
        findPreference<Preference>("load_state")?.setOnPreferenceClickListener {
            viewModel.loadState()
            playGame()
            true
        }
        findPreference<Preference>("close_game")?.setOnPreferenceClickListener {
            viewModel.closeGame()
            closeGameMenu()
            true
        }

        val nowPlaying = requireContext().resources.getString(R.string.main_menu_now_playing)
        title = nowPlaying

        observeNow(viewModel.loadedGame) { game ->
            if (game == null) return@observeNow

            title = "$nowPlaying: ${game.name}"

            val hasSaveState = game.currentState.exists()
            findPreference<Preference>("load_state")?.isEnabled = hasSaveState
        }
    }

    override fun onResume() {
        super.onResume()
        requireActivity().title = title
    }

    private fun playGame() {
        val intent = Intent(activity, GameActivity::class.java)
        startActivity(intent)
    }

    private fun closeGameMenu() {
        val main = activity as MainActivity
        main.closeAllSubMenus()
    }
}