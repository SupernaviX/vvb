package com.simongellis.vvb.menu

import android.app.Activity
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.os.Environment
import androidx.activity.result.contract.ActivityResultContract
import androidx.activity.result.contract.ActivityResultContracts.OpenDocument
import androidx.fragment.app.viewModels
import androidx.preference.Preference
import androidx.preference.PreferenceFragmentCompat
import com.nononsenseapps.filepicker.FilePickerActivity
import com.simongellis.vvb.MainViewModel
import com.simongellis.vvb.R
import com.simongellis.vvb.game.GameActivity

class MainMenuFragment: PreferenceFragmentCompat() {
    private val viewModel: MainViewModel by viewModels({ requireActivity() })

    companion object OpenPersistentDocument : OpenDocument() {
        override fun createIntent(context: Context, input: Array<out String>): Intent {
            return super.createIntent(context, input)
                .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                .addFlags(Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION)
        }
    }

    class OpenFilePicker : ActivityResultContract<Unit, Uri?>() {
        override fun createIntent(context: Context, input: Unit?): Intent {
            return Intent(context, FilePickerActivity::class.java)
                .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                .addFlags(Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION)
        }

        override fun parseResult(resultCode: Int, intent: Intent?): Uri? {
            if (resultCode != Activity.RESULT_OK) {
                return null
            }
            return intent?.data
        }
    }

    override fun onCreatePreferences(savedInstanceState: Bundle?, rootKey: String?) {
        setPreferencesFromResource(R.xml.preferences, rootKey)

        findPreference<Preference>("resume_game")?.setOnPreferenceClickListener {
            playGame()
            true
        }

        val chooseGameFilePicker = registerForActivityResult(OpenFilePicker(), this::loadGame)
        val chooseGameStorageFramework = registerForActivityResult(OpenPersistentDocument) { uri ->
            uri?.also {
                requireContext().contentResolver.takePersistableUriPermission(it, Intent.FLAG_GRANT_READ_URI_PERMISSION)
            }
            loadGame(uri)
        }
        findPreference<Preference>("load_game")?.setOnPreferenceClickListener {
            if (isFilePickerSupported()) {
                chooseGameFilePicker.launch(Unit)
            } else {
                chooseGameStorageFramework.launch(arrayOf("*/*"))
            }
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

    private fun loadGame(uri: Uri?) {
        uri?.also {
            if (viewModel.loadGame(it)) {
                playGame()
            }
        }
    }

    private fun playGame() {
        val intent = Intent(activity, GameActivity::class.java)
        startActivity(intent)
    }

    private fun isFilePickerSupported(): Boolean {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            // Android 11 and above can't access external storage files directly,
            // and they fail in ways that the below test can't easily detect
            return false
        }
        return try {
            // If we can read the filesystem at all, this device supports filepickers
            @Suppress("DEPRECATION")
            Environment.getExternalStorageDirectory().listFiles() != null
        } catch (_: Exception) {
            false
        }
    }
}