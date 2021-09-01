package com.simongellis.vvb.menu

import android.os.Bundle
import android.view.InputDevice
import android.view.KeyEvent
import android.view.MotionEvent
import android.view.View
import androidx.fragment.app.viewModels
import androidx.preference.PreferenceFragmentCompat
import com.simongellis.vvb.R
import com.simongellis.vvb.emulator.Input
import com.simongellis.vvb.utils.observe
import kotlin.math.absoluteValue

class ControllerInputMenuFragment: PreferenceFragmentCompat(), View.OnKeyListener, View.OnGenericMotionListener {
    private val viewModel: ControllerInputViewModel by viewModels()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        observe(viewModel.controller) {
            val defaultTitle = getText(R.string.main_menu_controller_setup)
            requireActivity().title = "$defaultTitle: ${it.name}"
        }
        viewModel.inputSummaries.forEach { (input, summary) ->
            observe(summary) {
                when (it) {
                    is ControllerInputViewModel.InputDisplay.Resource -> {
                        findPreference(input)?.setSummary(it.id)
                    }
                    is ControllerInputViewModel.InputDisplay.Text -> {
                        findPreference(input)?.summary = it.value
                    }
                }
            }
        }
    }

    override fun onCreatePreferences(savedInstanceState: Bundle?, rootKey: String?) {
        setPreferencesFromResource(R.xml.preferences_controller_input, rootKey)
        Input.values().forEach { configureInputPreference(it) }
    }

    private fun configureInputPreference(input: Input) {
        val pref = findPreference(input) ?: return
        pref.setOnClickListener {
            viewModel.startBinding(input, false)
            true
        }
        pref.setOnLongClickListener {
            viewModel.startBinding(input, true)
            true
        }
    }

    override fun onKey(v: View?, keyCode: Int, event: KeyEvent): Boolean {
        if (event.action != KeyEvent.ACTION_DOWN || event.repeatCount > 0) {
            return false
        }
        val device = InputDevice.getDevice(event.deviceId) ?: return false

        return viewModel.persistKeyMapping(device, keyCode)
    }

    override fun onGenericMotion(v: View?, event: MotionEvent): Boolean {
        val device = InputDevice.getDevice(event.deviceId) ?: return false
        val (axis, value) = device.motionRanges
            .filter { it.isFromSource(InputDevice.SOURCE_CLASS_JOYSTICK) }
            .map { it.axis to event.getAxisValue(it.axis) }
            .firstOrNull { (_, value) -> value.absoluteValue > 0.45 }
            ?: return false

        return viewModel.persistAxisMapping(device, axis, value < 0)
    }

    private fun findPreference(input: Input): ControllerInputPreference? {
        return findPreference(input.prefName)
    }
}