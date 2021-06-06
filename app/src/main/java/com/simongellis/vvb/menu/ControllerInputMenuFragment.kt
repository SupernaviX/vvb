package com.simongellis.vvb.menu

import android.hardware.input.InputManager
import android.os.Bundle
import android.view.InputDevice
import android.view.KeyEvent
import android.view.MotionEvent
import android.view.View
import androidx.core.content.ContextCompat.getSystemService
import androidx.preference.PreferenceFragmentCompat
import com.simongellis.vvb.R
import com.simongellis.vvb.emulator.Input
import com.simongellis.vvb.game.ControllerDao
import kotlin.math.absoluteValue

class ControllerInputMenuFragment: PreferenceFragmentCompat(), View.OnKeyListener, View.OnGenericMotionListener {
    private var _input: Input? = null
    private var _bindingMultiple = false

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
        val pref = findPreference(input) ?: return
        pref.setOnClickListener {
            _input = input
            pref.setIsBinding(false)
            _bindingMultiple = false
            true
        }
        pref.setOnLongClickListener {
            _input = input
            pref.setIsBinding(true)
            _bindingMultiple = true
            true
        }
        pref.setMappings(_controllerDao.getMappings(_id, input))
    }

    override fun onResume() {
        super.onResume()
        val defaultTitle = getText(R.string.main_menu_controller_setup)
        requireActivity().title = "$defaultTitle: $_name"
    }

    override fun onKey(v: View?, keyCode: Int, event: KeyEvent): Boolean {
        if (event.action != KeyEvent.ACTION_DOWN || event.repeatCount > 0) {
            return false
        }
        val input = _input ?: return false
        val device = _inputManager.getInputDevice(event.deviceId)

        val mapping = ControllerDao.KeyMapping(device.descriptor, input, keyCode)
        persistMapping(mapping)
        return true
    }

    override fun onGenericMotion(v: View?, event: MotionEvent): Boolean {
        val input = _input ?: return false
        val device = _inputManager.getInputDevice(event.deviceId)
        val (axis, value) = device.motionRanges
            .filter { it.isFromSource(InputDevice.SOURCE_CLASS_JOYSTICK) }
            .map { it.axis to event.getAxisValue(it.axis) }
            .firstOrNull { (_, value) -> value.absoluteValue > 0.45 }
            ?: return false

        val mapping = ControllerDao.AxisMapping(device.descriptor, input, axis, value < 0)
        persistMapping(mapping)
        return true
    }

    private fun persistMapping(mapping: ControllerDao.Mapping) {
        if (_bindingMultiple) {
            _controllerDao.addMapping(_id, mapping)
        } else {
            _controllerDao.putMapping(_id, mapping)
        }
        findPreference(mapping.input)?.setMappings(_controllerDao.getMappings(_id, mapping.input))
        _input = null
    }

    private fun findPreference(input: Input): ControllerInputPreference? {
        return findPreference(input.prefName)
    }
}