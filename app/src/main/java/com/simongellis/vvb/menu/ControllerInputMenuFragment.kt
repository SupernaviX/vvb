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
import java.util.*

class ControllerInputMenuFragment: PreferenceFragmentCompat(), Preference.OnPreferenceClickListener, View.OnKeyListener {
    private var _control: String? = null
    private lateinit var _sharedPreferences: SharedPreferences
    private lateinit var _inputManager: InputManager
    private lateinit var _controllerId: String
    private lateinit var _controllerName: String

    override fun onCreate(savedInstanceState: Bundle?) {
        _sharedPreferences = PreferenceManager.getDefaultSharedPreferences(context)
        _inputManager = getSystemService(requireContext(), InputManager::class.java)!!

        val (id, name) = getControllerDescriptor().split("::", limit = 2)
        _controllerId = id
        _controllerName = name

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
        if (_sharedPreferences.contains(preferenceKey(input))) {
            pref.summary = "Mapped"
        } else {
            pref.summary = "Unmapped"
        }
    }

    override fun onResume() {
        super.onResume()
        val defaultTitle = getText(R.string.main_menu_controller_input_setup)
        requireActivity().title = "$defaultTitle: $_controllerName"
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
        val input = "button::${device}_$keyCode"
        _sharedPreferences.edit {
            putString(preferenceKey(_control!!), input)
        }
        _control = null
        return true
    }

    private fun getControllerDescriptor(): String {
        val controllers = _sharedPreferences.getStringSet("controllers", setOf())!!
        if (controllers.isNotEmpty()) {
            return controllers.first()
        }
        val descriptor = "${UUID.randomUUID()}::Controller 1"
        _sharedPreferences.edit().putStringSet("controllers", setOf(descriptor)).apply()
        return descriptor
    }

    private fun preferenceKey(input: String): String {
        return "controller_${_controllerId}_$input"
    }
}