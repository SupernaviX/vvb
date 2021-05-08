package com.simongellis.vvb.menu

import android.content.SharedPreferences
import android.hardware.input.InputManager
import android.os.Bundle
import android.view.KeyEvent
import android.view.View
import androidx.core.content.ContextCompat.getSystemService
import androidx.core.content.edit
import androidx.preference.Preference
import androidx.preference.PreferenceFragmentCompat
import androidx.preference.PreferenceManager
import com.simongellis.vvb.R
import com.simongellis.vvb.emulator.Input

class InputMenuFragment: PreferenceFragmentCompat(), Preference.OnPreferenceClickListener, View.OnKeyListener {
    private var _control: String? = null
    private lateinit var _sharedPreferences: SharedPreferences
    private lateinit var _inputManager: InputManager

    override fun onCreate(savedInstanceState: Bundle?) {
        _sharedPreferences = PreferenceManager.getDefaultSharedPreferences(context)
        _inputManager = getSystemService(requireContext(), InputManager::class.java)!!
        super.onCreate(savedInstanceState)
    }

    override fun onCreatePreferences(savedInstanceState: Bundle?, rootKey: String?) {
        setPreferencesFromResource(R.xml.preferences_input, rootKey)
        Input.values().forEach { input ->
            input.prefName?.also { configureInputPreference(it) }
        }
    }

    private fun configureInputPreference(key: String) {
        val pref = findPreference<Preference>(key)!!
        pref.onPreferenceClickListener = this
        if (_sharedPreferences.contains(key)) {
            pref.summary = "Mapped"
        } else {
            pref.summary = "Unmapped"
        }
    }

    override fun onResume() {
        super.onResume()
        requireActivity().setTitle(R.string.main_menu_input_setup)
    }

    override fun onPreferenceClick(preference: Preference): Boolean {
        // Start mapping a control
        _control = preference.key
        preference.summary = "Press any key..."
        return true
    }

    override fun onKey(v: View?, keyCode: Int, event: KeyEvent): Boolean {
        if (_control == null || event.action != KeyEvent.ACTION_DOWN) {
            return false
        }

        // We have a control and an input event,
        // so persist a mapping between the two
        findPreference<Preference>(_control!!)?.summary = "Mapped"
        val device = _inputManager.getInputDevice(event.deviceId)?.descriptor
        val input = "$device::$keyCode"
        _sharedPreferences.edit {
            putString(_control, input)
        }
        _control = null
        return true
    }
}