package com.simongellis.vvb.menu

import android.net.Uri
import android.os.Bundle
import android.widget.Toast
import androidx.preference.Preference
import androidx.preference.PreferenceFragmentCompat
import com.simongellis.vvb.R

class GeneralMenuFragment: PreferenceFragmentCompat() {
    private val exportFolderPicker = FilePicker(this, FilePicker.Mode.Write("vvb.zip", "application/zip"), ::exportData)
    private val importFolderPicker = FilePicker(this, FilePicker.Mode.Read, ::importData)

    override fun onCreatePreferences(savedInstanceState: Bundle?, rootKey: String?) {
        setPreferencesFromResource(R.xml.preferences_general, rootKey)

        findPreference<Preference>("export_data")?.setOnPreferenceClickListener {
            exportFolderPicker.open()
            true
        }
        findPreference<Preference>("import_data")?.setOnPreferenceClickListener {
            importFolderPicker.open()
            true
        }
    }

    override fun onResume() {
        super.onResume()
        requireActivity().setTitle(R.string.main_menu_general_setup)
    }

    private fun exportData(uri: Uri?) {
        Toast.makeText(context, "Export to ${uri?.path}", Toast.LENGTH_LONG).show()
    }

    private fun importData(uri: Uri?) {
        Toast.makeText(context, "Import from ${uri?.path}", Toast.LENGTH_LONG).show()
    }
}