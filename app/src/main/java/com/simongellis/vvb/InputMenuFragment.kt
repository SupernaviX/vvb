package com.simongellis.vvb

import android.content.SharedPreferences
import android.hardware.input.InputManager
import android.os.Bundle
import android.util.Log
import android.view.KeyEvent
import androidx.core.content.ContextCompat.getSystemService
import androidx.core.content.edit
import androidx.preference.Preference
import androidx.preference.PreferenceFragmentCompat
import androidx.preference.PreferenceManager

class InputMenuFragment: PreferenceFragmentCompat(), Preference.OnPreferenceClickListener {
    private var _control: String? = null
    private lateinit var _sharedPreferences: SharedPreferences
    private lateinit var _inputManager: InputManager

    override fun onCreate(savedInstanceState: Bundle?) {
        _sharedPreferences = PreferenceManager.getDefaultSharedPreferences(context)
        _inputManager = getSystemService(context!!, InputManager::class.java)!!
        super.onCreate(savedInstanceState)
    }

    override fun onCreatePreferences(savedInstanceState: Bundle?, rootKey: String?) {
        addPreferencesFromResource(R.xml.preferences_input)
        Input.values().forEach { input ->
            configureInputPreference(input.prefName)
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
        activity!!.setTitle(R.string.main_menu_input_setup)
    }

    override fun onPreferenceClick(preference: Preference): Boolean {
        // Start mapping a control
        _control = preference.key
        preference.summary = "Press any key..."
        return true
    }

    fun onKeyEvent(event: KeyEvent): Boolean {
        if (_control == null || event.action != KeyEvent.ACTION_DOWN) {
            return false
        }

        // We have a control and an input event,
        // so persist a mapping between the two
        findPreference<Preference>(_control!!)?.summary = "Mapped"
        val device = _inputManager.getInputDevice(event.deviceId)?.descriptor
        val keyCode = event.keyCode
        val input = "$device::$keyCode"
        _sharedPreferences.edit {
            putString(_control, input)
        }
        _control = null
        return true
    }
}