package com.simongellis.vvb

import android.hardware.input.InputManager
import android.os.Bundle
import androidx.core.content.ContextCompat.getSystemService
import androidx.preference.Preference
import androidx.preference.PreferenceFragmentCompat

class InputMenuFragment: PreferenceFragmentCompat() {
    override fun onCreatePreferences(savedInstanceState: Bundle?, rootKey: String?) {
        val context = preferenceManager.context
        val screen = preferenceManager.createPreferenceScreen(context)

        val inputManager = getSystemService(context, InputManager::class.java)!!
        inputManager.inputDeviceIds.forEach {
            val device = inputManager.getInputDevice(it)!!
            val preference = Preference(context).apply {
                key = device.descriptor
                title = device.name
            }
            screen.addPreference(preference)
        }

        preferenceScreen = screen
    }

    override fun onResume() {
        super.onResume()
        activity!!.setTitle(R.string.main_menu_input_setup)
    }
}