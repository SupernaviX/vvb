package com.simongellis.vvb.game

import android.content.Context
import android.hardware.input.InputManager
import android.view.KeyEvent
import androidx.core.content.ContextCompat.getSystemService
import com.simongellis.vvb.emulator.Input
import kotlin.collections.HashMap

class InputBindingMapper(context: Context): InputManager.InputDeviceListener {
    data class Binding(val deviceId: Int, val keyCode: Int)

    private val _bindings = HashMap<Binding, Input>()

    private val _controllerPrefs = ControllerPreferences(context)
    private val _inputManager = getSystemService(context, InputManager::class.java)!!

    init {
        _inputManager.registerInputDeviceListener(this, null)

        val devices = _inputManager.inputDeviceIds
            .map { _inputManager.getInputDevice(it).descriptor to it }
            .toMap()

        _controllerPrefs.mappings.forEach { mapping ->
            // Prepare the binding for this input
            devices[mapping.device]?.also { deviceId ->
                val binding = Binding(deviceId, mapping.keyCode)
                _bindings[binding] = mapping.input
            }
        }
    }

    fun destroy() {
        _inputManager.unregisterInputDeviceListener(this)
    }

    fun getBoundInput(event: KeyEvent): Input? {
        return _bindings[Binding(event.deviceId, event.keyCode)]
    }

    override fun onInputDeviceAdded(newDeviceId: Int) {
        // Prepare any bindings which should be attached to this device
        val device = _inputManager.getInputDevice(newDeviceId)
        _controllerPrefs.deviceMappings[device.descriptor]?.forEach { mapping ->
            val binding = Binding(newDeviceId, mapping.keyCode)
            _bindings[binding] = mapping.input
        }
    }

    override fun onInputDeviceRemoved(oldDeviceId: Int) {
        // Remove any bindings which were attached to this device
        _bindings.keys
            .filter { (deviceId) -> deviceId == oldDeviceId }
            .forEach { _bindings.remove(it) }
    }

    override fun onInputDeviceChanged(deviceId: Int) {
        // no-op
    }
}