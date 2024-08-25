package com.simongellis.vvb.menu

import android.app.Activity
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.Environment
import androidx.activity.result.contract.ActivityResultContract
import androidx.fragment.app.Fragment
import com.nononsenseapps.filepicker.FilePickerActivity

class FilePicker(fragment: Fragment, mode: Mode, onFileChosen: (uri: Uri?) -> Unit) {
    sealed interface Mode {
        object Read: Mode
        class Write(val title: String, val mimeType: String): Mode
    }
    private val launcherFilePicker = fragment.registerForActivityResult(filePickerContract(mode), onFileChosen)
    private val launcherReadStorageFramework = fragment.registerForActivityResult(storageFrameworkContract(mode)) { uri ->
        uri?.also {
            if (it.scheme == "content") {
                fragment.requireContext().contentResolver.takePersistableUriPermission(it, Intent.FLAG_GRANT_READ_URI_PERMISSION)
            }
        }
        onFileChosen(uri)
    }

    fun open() {
        if (isFilePickerSupported()) {
            launcherFilePicker.launch(Unit)
        } else {
            launcherReadStorageFramework.launch(Unit)
        }
    }

    companion object {
        fun isFilePickerSupported(): Boolean {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
                // Android 11 and above can't access external storage files directly,
                // and they fail in ways that the below test can't easily detect
                return false
            }
            return try {
                // If we can read the filesystem at all, this device supports filepickers
                Environment.getExternalStorageDirectory().listFiles() != null
            } catch (_: Exception) {
                false
            }
        }

        private fun storageFrameworkContract(mode: Mode): ActivityResultContract<Unit, Uri?> {
            return when (mode) {
                is Mode.Read -> ReadStorageFramework
                is Mode.Write -> WriteStorageFramework(mode.title, mode.mimeType)
            }
        }

        private fun filePickerContract(mode: Mode): ActivityResultContract<Unit, Uri?> {
            return when (mode) {
                is Mode.Read -> ReadFilePicker
                is Mode.Write -> WriteFilePicker
            }
        }

        private object ReadStorageFramework : ActivityResultContract<Unit, Uri?>() {
            override fun createIntent(context: Context, input: Unit): Intent {
                return Intent(Intent.ACTION_OPEN_DOCUMENT)
                    .setType("*/*")
                    .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                    .addFlags(Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION)
            }

            override fun parseResult(resultCode: Int, intent: Intent?): Uri? {
                return intent.takeIf { resultCode == Activity.RESULT_OK }?.data
            }
        }

        private object ReadFilePicker : ActivityResultContract<Unit, Uri?>() {
            override fun createIntent(context: Context, input: Unit): Intent {
                return Intent(context, FilePickerActivity::class.java)
                    .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                    .addFlags(Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION)
            }

            override fun parseResult(resultCode: Int, intent: Intent?): Uri? {
                return intent.takeIf { resultCode == Activity.RESULT_OK }?.data
            }
        }

        private class WriteStorageFramework(private val title: String, private val mimeType: String) : ActivityResultContract<Unit, Uri?>() {
            override fun createIntent(context: Context, input: Unit): Intent {
                return Intent(Intent.ACTION_CREATE_DOCUMENT)
                    .setType(mimeType)
                    .putExtra(Intent.EXTRA_TITLE, title)
                    .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                    .addFlags(Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION)
            }

            override fun parseResult(resultCode: Int, intent: Intent?): Uri? {
                return intent.takeIf { resultCode == Activity.RESULT_OK }?.data
            }
        }

        private object WriteFilePicker : ActivityResultContract<Unit, Uri?>() {
            override fun createIntent(context: Context, input: Unit): Intent {
                return Intent(context, FilePickerActivity::class.java)
                    .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                    .addFlags(Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION)
                    .putExtra(FilePickerActivity.EXTRA_MODE, FilePickerActivity.MODE_NEW_FILE)
                    .putExtra(FilePickerActivity.EXTRA_ALLOW_EXISTING_FILE, true)
            }

            override fun parseResult(resultCode: Int, intent: Intent?): Uri? {
                if (resultCode != Activity.RESULT_OK) {
                    return null
                }
                return intent?.data
            }
        }
    }
}