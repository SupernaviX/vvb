package com.simongellis.vvb.menu

import android.content.Context
import android.content.Intent
import android.os.Bundle
import androidx.activity.result.contract.ActivityResultContracts.OpenDocument
import androidx.fragment.app.viewModels
import androidx.preference.Preference
import androidx.preference.PreferenceFragmentCompat
import com.simongellis.vvb.MainViewModel
import com.simongellis.vvb.R
import com.simongellis.vvb.game.GameActivity

class MainMenuFragment: PreferenceFragmentCompat() {
    private val viewModel: MainViewModel by viewModels()

    companion object OpenPersistentDocument : OpenDocument() {
        override fun createIntent(context: Context, input: Array<out String>): Intent {
            return super.createIntent(context, input)
                .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                .addFlags(Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION)
        }
    }

    override fun onCreatePreferences(savedInstanceState: Bundle?, rootKey: String?) {
        setPreferencesFromResource(R.xml.preferences, rootKey)

        findPreference<Preference>("resume_game")?.setOnPreferenceClickListener {
            playGame()
            true
        }

        val chooseGame = registerForActivityResult(OpenPersistentDocument) { uri ->
            uri?.also {
                if (viewModel.loadGame(it)) {
                    playGame()
                }
            }
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
            isVisible = viewModel.isGameLoaded
        }
    }

    private fun playGame() {
        val intent = Intent(activity, GameActivity::class.java)
        startActivity(intent)
    }
}