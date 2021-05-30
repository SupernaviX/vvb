package com.simongellis.vvb.menu

import android.hardware.input.InputManager
import android.os.Bundle
import android.view.KeyEvent
import android.view.View
import androidx.core.content.ContextCompat.getSystemService
import androidx.core.content.edit
import androidx.preference.Preference
import androidx.preference.PreferenceFragmentCompat
import com.simongellis.vvb.R
import com.simongellis.vvb.emulator.Input

class ControllerInputMenuFragment: PreferenceFragmentCompat(), Preference.OnPreferenceClickListener, View.OnKeyListener {
    private var _control: String? = null
    private lateinit var _inputManager: InputManager
    private lateinit var _id: String
    private lateinit var _name: String

    override fun onCreate(savedInstanceState: Bundle?) {
        _inputManager = getSystemService(requireContext(), InputManager::class.java)!!
        _id = requireArguments().getString("id")!!
        _name = requireArguments().getString("name")!!

        super.onCreate(savedInstanceState)
    }

    override fun onCreatePreferences(savedInstanceState: Bundle?, rootKey: String?) {
        setPreferencesFromResource(R.xml.preferences_controller_input, rootKey)
        Input.values().forEach { input ->
            input.prefName?.also { configureInputPreference(it) }
        }
    }

    private fun configureInputPreference(input: String) {
        val pref = findPreference<Preference>(input)!!
        pref.onPreferenceClickListener = this
        if (preferenceManager.sharedPreferences.contains(preferenceKey(input))) {
            pref.setSummary(R.string.input_menu_mapped)
        } else {
            pref.setSummary(R.string.input_menu_unmapped)
        }
    }

    override fun onResume() {
        super.onResume()
        val defaultTitle = getText(R.string.main_menu_controller_setup)
        requireActivity().title = "$defaultTitle: $_name"
    }

    override fun onPreferenceClick(preference: Preference): Boolean {
        // Start mapping a control
        _control = preference.key
        preference.setSummary(R.string.input_menu_press_any_key)
        return true
    }

    override fun onKey(v: View?, keyCode: Int, event: KeyEvent): Boolean {
        if (_control == null || event.action != KeyEvent.ACTION_DOWN) {
            return false
        }

        // We have a control and an input event,
        // so persist a mapping between the two
        findPreference<Preference>(_control!!)?.setSummary(R.string.input_menu_mapped)
        val device = _inputManager.getInputDevice(event.deviceId)?.descriptor
        val input = "button::${device}_$keyCode"
        preferenceManager.sharedPreferences.edit {
            putString(preferenceKey(_control!!), input)
        }
        _control = null
        return true
    }

    private fun preferenceKey(input: String): String {
        return "controller_${_id}_$input"
    }
}