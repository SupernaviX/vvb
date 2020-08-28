package com.simongellis.vvb

import android.content.Context
import android.hardware.input.InputManager
import android.view.KeyEvent
import androidx.core.content.ContextCompat.getSystemService
import androidx.preference.PreferenceManager
import kotlin.collections.HashMap

class InputBindingMapper(context: Context): InputManager.InputDeviceListener {
    data class Binding(val deviceId: Int, val keyCode: Int)

    private val _inputManager: InputManager
    private val _deviceControls = HashMap<String, HashMap<Int, Input>>()
    private val _bindings = HashMap<Binding, Input>()

    init {
        val prefs = PreferenceManager.getDefaultSharedPreferences(context)
        _inputManager = getSystemService(context, InputManager::class.java)!!
        _inputManager.registerInputDeviceListener(this, null)

        val devices = _inputManager.inputDeviceIds
            .map { _inputManager.getInputDevice(it).descriptor to it }
            .toMap()

        Input.values().forEach { input ->
            val savedBinding = prefs.getString(input.prefName, null)
            if (savedBinding != null) {
                val (device, keyCodeStr) = savedBinding.split("::")
                val keyCode = keyCodeStr.toInt(10)

                // Remember which device this mapping is bound to
                _deviceControls.getOrPut(device) { HashMap() }[keyCode] = input

                // Prepare the binding for this input
                devices[device]?.also { deviceId ->
                    val binding = Binding(deviceId, keyCode)
                    _bindings[binding] = input
                }
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
        _deviceControls[device.descriptor]?.forEach { (keyCode, input) ->
            val binding = Binding(newDeviceId, keyCode)
            _bindings[binding] = input
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