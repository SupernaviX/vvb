package com.simongellis.vvb.menu

import android.content.Intent
import android.os.Bundle
import androidx.fragment.app.viewModels
import androidx.preference.Preference
import androidx.preference.PreferenceFragmentCompat
import com.simongellis.vvb.MainViewModel
import com.simongellis.vvb.R
import com.simongellis.vvb.data.Game
import com.simongellis.vvb.game.GameActivity
import com.simongellis.vvb.utils.observeNow

class RecentGamesMenuFragment: PreferenceFragmentCompat() {
    private val viewModel: MainViewModel by viewModels({ requireActivity() })

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        observeNow(viewModel.recentGames) { updateRecentGames(it) }
    }

    override fun onResume() {
        super.onResume()
        requireActivity().setTitle(R.string.main_menu_recent_games)
    }

    override fun onCreatePreferences(savedInstanceState: Bundle?, rootKey: String?) {
        preferenceScreen = preferenceManager.createPreferenceScreen(context)
    }

    private fun updateRecentGames(recentGames: List<Game>) {
        // This method is triggered on preference change, so it can run after the fragment dies.
        // Bail early if this has happened to avoid calamity
        val context = context ?: return

        preferenceScreen.removeAll()
        for (game in recentGames) {
            preferenceScreen.addPreference(Preference(context).apply {
                key = game.id
                title = game.name
                setOnPreferenceClickListener {
                    if (viewModel.loadGame(game.uri)) {
                        playGame()
                    }
                    true
                }
            })
        }
    }

    private fun playGame() {
        val intent = Intent(activity, GameActivity::class.java)
        startActivity(intent)
    }
}