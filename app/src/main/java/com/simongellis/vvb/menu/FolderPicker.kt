package com.simongellis.vvb.menu

import android.app.Activity
import android.content.Context
import android.content.Intent
import android.net.Uri
import androidx.activity.result.contract.ActivityResultContract
import androidx.activity.result.contract.ActivityResultContracts
import androidx.fragment.app.Fragment
import com.nononsenseapps.filepicker.FilePickerActivity

class FolderPicker(fragment: Fragment, onFolderChosen: (uri: Uri?) -> Unit) {
    private val launcherFilePicker = fragment.registerForActivityResult(OpenFilePicker, onFolderChosen)
    private val launcherStorageFramework = fragment.registerForActivityResult(OpenPersistentDocumentTree) { uri ->
        uri?.also {
            if (it.scheme == "content") {
                fragment.requireContext().contentResolver.takePersistableUriPermission(it, Intent.FLAG_GRANT_READ_URI_PERMISSION.or(Intent.FLAG_GRANT_WRITE_URI_PERMISSION))
            }
        }
        onFolderChosen(uri)
    }

    fun open() {
        if (FilePicker.isFilePickerSupported()) {
            launcherFilePicker.launch(null)
        } else {
            launcherStorageFramework.launch(null)
        }
    }

    companion object {
        private object OpenPersistentDocumentTree : ActivityResultContracts.OpenDocumentTree() {
            override fun createIntent(context: Context, input: Uri?): Intent {
                return super.createIntent(context, input)
                    .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                    .addFlags(Intent.FLAG_GRANT_WRITE_URI_PERMISSION)
                    .addFlags(Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION)
            }
        }

        private object OpenFilePicker : ActivityResultContract<Unit, Uri?>() {
            override fun createIntent(context: Context, input: Unit): Intent {
                return Intent(context, FilePickerActivity::class.java)
                    .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                    .addFlags(Intent.FLAG_GRANT_WRITE_URI_PERMISSION)
                    .addFlags(Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION)
                    .putExtra(FilePickerActivity.EXTRA_MODE, FilePickerActivity.MODE_DIR)
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