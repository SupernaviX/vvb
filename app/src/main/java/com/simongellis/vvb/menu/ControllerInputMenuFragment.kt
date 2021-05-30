package com.simongellis.vvb.menu

import android.hardware.input.InputManager
import android.os.Bundle
import android.view.InputDevice
import android.view.KeyEvent
import android.view.MotionEvent
import android.view.View
import androidx.core.content.ContextCompat.getSystemService
import androidx.core.content.edit
import androidx.preference.Preference
import androidx.preference.PreferenceFragmentCompat
import com.simongellis.vvb.R
import com.simongellis.vvb.emulator.Input
import kotlin.math.absoluteValue

class ControllerInputMenuFragment: PreferenceFragmentCompat(), Preference.OnPreferenceClickListener, View.OnKeyListener, View.OnGenericMotionListener {
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
        if (preferenceManager.sharedPreferences.contains(mappingKey(input))) {
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
        preference.setSummary(R.string.input_menu_press_any_input)
        return true
    }

    override fun onKey(v: View?, keyCode: Int, event: KeyEvent): Boolean {
        if (_control == null || event.action != KeyEvent.ACTION_DOWN) {
            return false
        }
        val device = _inputManager.getInputDevice(event.deviceId)

        // We have a control and an input event,
        // so persist a mapping between the two
        val mapping = "${device.descriptor}::key::$keyCode"
        preferenceManager.sharedPreferences.edit {
            putString(mappingKey(_control!!), mapping)
        }
        findPreference<Preference>(_control!!)?.setSummary(R.string.input_menu_mapped)
        _control = null
        return true
    }

    override fun onGenericMotion(v: View?, event: MotionEvent): Boolean {
        if (_control == null) {
            return false
        }
        val device = _inputManager.getInputDevice(event.deviceId)
        val (axis, value) = device.motionRanges
            .filter { it.isFromSource(InputDevice.SOURCE_CLASS_JOYSTICK) }
            .map { it.axis to event.getAxisValue(it.axis) }
            .firstOrNull { (_, value) -> value.absoluteValue > 0.45 }
            ?: return false

        // We have a control and an active axis,
        // so persist a mapping between the two
        val sign = if (value < 0) { '-' } else { '+' }
        val mapping = "${device.descriptor}::axis::${axis}_${sign}"
        preferenceManager.sharedPreferences.edit {
            putString(mappingKey(_control!!), mapping)
        }
        findPreference<Preference>(_control!!)?.setSummary(R.string.input_menu_mapped)
        _control = null
        return true
    }

    private fun mappingKey(input: String): String {
        return "controller_${_id}_$input"
    }
}