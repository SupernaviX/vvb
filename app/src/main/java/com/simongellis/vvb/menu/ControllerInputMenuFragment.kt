package com.simongellis.vvb.menu

import android.hardware.input.InputManager
import android.os.Bundle
import android.view.InputDevice
import android.view.KeyEvent
import android.view.MotionEvent
import android.view.View
import androidx.core.content.ContextCompat.getSystemService
import androidx.preference.Preference
import androidx.preference.PreferenceFragmentCompat
import com.simongellis.vvb.R
import com.simongellis.vvb.emulator.Input
import com.simongellis.vvb.game.ControllerDao
import kotlin.math.absoluteValue

class ControllerInputMenuFragment: PreferenceFragmentCompat(), Preference.OnPreferenceClickListener, View.OnKeyListener, View.OnGenericMotionListener {
    private var _control: String? = null
    private val _inputManager by lazy {
        getSystemService(requireContext(), InputManager::class.java)!!
    }
    private val _controllerDao by lazy {
        ControllerDao(preferenceManager.sharedPreferences)
    }
    private lateinit var _id: String
    private lateinit var _name: String

    override fun onCreate(savedInstanceState: Bundle?) {
        _id = requireArguments().getString("id")!!
        super.onCreate(savedInstanceState)
        _name = _controllerDao.getController(_id).name
    }

    override fun onCreatePreferences(savedInstanceState: Bundle?, rootKey: String?) {
        setPreferencesFromResource(R.xml.preferences_controller_input, rootKey)
        Input.values().forEach { configureInputPreference(it) }
    }

    private fun configureInputPreference(input: Input) {
        val pref = input.prefName?.let { findPreference<Preference>(it) } ?: return
        pref.onPreferenceClickListener = this
        if (_controllerDao.hasMapping(_id, input)) {
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

        val input = Input.values().find { it.prefName == _control }!!
        val mapping = ControllerDao.KeyMapping(device.descriptor, input, keyCode)
        persistMapping(mapping)
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

        val input = Input.values().find { it.prefName == _control }!!
        val mapping = ControllerDao.AxisMapping(device.descriptor, input, axis, value < 0)
        persistMapping(mapping)
        return true
    }

    private fun persistMapping(mapping: ControllerDao.Mapping) {
        _controllerDao.addMapping(_id, mapping)
        findPreference<Preference>(_control!!)?.setSummary(R.string.input_menu_mapped)
        _control = null
    }
}