package com.simongellis.vvb.menu

import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.provider.OpenableColumns
import androidx.fragment.app.viewModels
import androidx.preference.Preference
import androidx.preference.PreferenceFragmentCompat
import com.simongellis.vvb.MainViewModel
import com.simongellis.vvb.R
import com.simongellis.vvb.game.GameActivity

class RecentGamesMenuFragment: PreferenceFragmentCompat() {
    private val viewModel: MainViewModel by viewModels()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        viewModel.recentGames.observe(this, this::updateRecentGames)
    }

    override fun onResume() {
        super.onResume()
        requireActivity().setTitle(R.string.main_menu_recent_games)
    }

    override fun onCreatePreferences(savedInstanceState: Bundle?, rootKey: String?) {
        preferenceScreen = preferenceManager.createPreferenceScreen(context)
    }

    private fun updateRecentGames(recentGames: List<RecentGamesDao.RecentGame>) {
        // This method is triggered on preference change, so it can run after the fragment dies.
        // Bail early if this has happened to avoid calamity
        val context = context ?: return

        preferenceScreen.removeAll()
        for (recentGame in recentGames) {
            val uri = recentGame.uri
            preferenceScreen.addPreference(Preference(context).apply {
                key = uri.toString()
                title = getFilename(uri)
                setOnPreferenceClickListener {
                    if (viewModel.loadGame(uri)) {
                        playGame()
                    }
                    true
                }
            })
        }
    }

    private fun getFilename(uri: Uri): String {
        return requireContext().contentResolver.query(uri, null, null, null, null)
            ?.use {
                if (it.moveToFirst()) {
                    it.getString(it.getColumnIndex(OpenableColumns.DISPLAY_NAME))
                } else {
                    null
                }
            } ?: uri.toString()
    }

    private fun playGame() {
        val intent = Intent(activity, GameActivity::class.java)
        startActivity(intent)
    }
}