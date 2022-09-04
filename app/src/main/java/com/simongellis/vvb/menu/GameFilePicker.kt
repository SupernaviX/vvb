package com.simongellis.vvb.menu

import android.app.Activity
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.Environment
import androidx.activity.result.contract.ActivityResultContract
import androidx.activity.result.contract.ActivityResultContracts
import androidx.fragment.app.Fragment
import com.nononsenseapps.filepicker.FilePickerActivity

class GameFilePicker(fragment: Fragment, loadGame: (uri: Uri?) -> Unit) {
    private val chooseGameFilePicker = fragment.registerForActivityResult(OpenFilePicker(), loadGame)
    private val chooseGameStorageFramework = fragment.registerForActivityResult(OpenPersistentDocument) { uri ->
        uri?.also {
            if (it.scheme == "content") {
                fragment.requireContext().contentResolver.takePersistableUriPermission(it, Intent.FLAG_GRANT_READ_URI_PERMISSION)
            }
        }
        loadGame(uri)
    }

    fun open() {
        if (isFilePickerSupported()) {
            chooseGameFilePicker.launch(Unit)
        } else {
            chooseGameStorageFramework.launch(arrayOf("*/*"))
        }
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

    private companion object OpenPersistentDocument : ActivityResultContracts.OpenDocument() {
        override fun createIntent(context: Context, input: Array<String>): Intent {
            return super.createIntent(context, input)
                .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                .addFlags(Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION)
        }
    }

    private class OpenFilePicker : ActivityResultContract<Unit, Uri?>() {
        override fun createIntent(context: Context, input: Unit): Intent {
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
}