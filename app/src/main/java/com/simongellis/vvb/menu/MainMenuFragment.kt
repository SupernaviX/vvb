package com.simongellis.vvb.menu

import android.os.Bundle
import androidx.fragment.app.viewModels
import androidx.preference.Preference
import androidx.preference.PreferenceFragmentCompat
import com.simongellis.vvb.MainViewModel
import com.simongellis.vvb.R
import com.simongellis.vvb.utils.observeNow

class MainMenuFragment: PreferenceFragmentCompat() {
    private val viewModel: MainViewModel by viewModels({ requireActivity() })

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        observeNow(viewModel.loadedGame) { game ->
            findPreference<Preference>("game_actions")?.apply {
                isVisible = game != null
                if (game != null) {
                    val nowPlaying = context.resources.getString(R.string.main_menu_now_playing)
                    summary = "$nowPlaying: ${game.name}"
                }
            }
        }
    }

    override fun onCreatePreferences(savedInstanceState: Bundle?, rootKey: String?) {
        setPreferencesFromResource(R.xml.preferences, rootKey)
    }

    override fun onResume() {
        super.onResume()
        requireActivity().setTitle(R.string.app_name)
        if (viewModel.lastEvent.compareAndSet(MainViewModel.GameEvent.Closed, null)) {
            // if we just closed a game, hide the "game_actions" menu
            findPreference<Preference>("game_actions")?.isVisible = false
        }
    }
}