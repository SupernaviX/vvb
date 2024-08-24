package com.simongellis.vvb.menu

import android.os.Bundle
import androidx.preference.PreferenceFragmentCompat
import com.simongellis.vvb.R

class GeneralMenuFragment: PreferenceFragmentCompat() {
    override fun onCreatePreferences(savedInstanceState: Bundle?, rootKey: String?) {
        setPreferencesFromResource(R.xml.preferences_general, rootKey)

        val externalDirectoryPreference = findPreference<FolderPreference>("data_storage_external_directory")
        externalDirectoryPreference?.initialize(this@GeneralMenuFragment)

        findPreference<DetailedListPreference>("data_storage_location")?.apply {
            detailedEntries = listOf(
                DetailedListPreference.Entry("internal", getString(R.string.data_storage_location_internal_summary), getString(R.string.data_storage_location_internal_description)),
                DetailedListPreference.Entry("external", getString(R.string.data_storage_location_external_summary), getString(R.string.data_storage_location_external_description)),
            )
            if (value == null) {
                value = detailedEntries[0].value
            }
            externalDirectoryPreference?.isVisible = value == "external"
            setOnPreferenceChangeListener { _, newValue ->
                externalDirectoryPreference?.isVisible = newValue == "external"
                true
            }
        }
    }

    override fun onResume() {
        super.onResume()
        requireActivity().setTitle(R.string.main_menu_general_setup)
    }
}