package com.simongellis.vvb.game

import android.content.Context
import android.hardware.input.InputManager
import android.view.KeyEvent
import android.view.MotionEvent
import androidx.core.content.ContextCompat.getSystemService
import com.simongellis.vvb.emulator.Input
import kotlin.collections.HashMap

class InputBindingMapper(context: Context): InputManager.InputDeviceListener {
    data class AxisBinding(val axis: Int, val isNegative: Boolean, val input: Input)

    class DeviceBindings(mappings: List<ControllerDao.Mapping>) {
        val keyBindings = mappings
            .filterIsInstance<ControllerDao.KeyMapping>()
            .map { it.keyCode to it.input }
            .toMap()
        val axisBindings = mappings
            .filterIsInstance<ControllerDao.AxisMapping>()
            .map { AxisBinding(it.axis, it.isNegative, it.input) }
    }

    private val _deviceMappings = ControllerDao(context)
        .getAllMappings()
        .groupBy { it.device }

    private val _deviceBindings = HashMap<Int, DeviceBindings>()
    private val _inputManager = getSystemService(context, InputManager::class.java)!!

    init {
        _inputManager.registerInputDeviceListener(this, null)
        _inputManager.inputDeviceIds.forEach {
            onInputDeviceAdded(it)
        }
    }

    fun destroy() {
        _inputManager.unregisterInputDeviceListener(this)
    }

    fun getBoundInput(event: KeyEvent): Input? {
        val keyBindings = _deviceBindings[event.deviceId]?.keyBindings
            ?: return null
        return keyBindings[event.keyCode]
    }

    // returns one list of all "pressed" axis inputs, and one list of all released ones
    fun getAxisInputs(event: MotionEvent): Pair<List<Input>, List<Input>> {
        val pressed = mutableListOf<Input>()
        val released = mutableListOf<Input>()

        _deviceBindings[event.deviceId]?.axisBindings?.also {
            for (binding in it) {
                val axisValue = event.getAxisValue(binding.axis)
                if (binding.isNegative && axisValue < -0.45) {
                    pressed.add(binding.input)
                } else if (!binding.isNegative && axisValue > 0.45) {
                    pressed.add(binding.input)
                } else {
                    released.add(binding.input)
                }
            }
        }

        return pressed to released
    }

    override fun onInputDeviceAdded(newDeviceId: Int) {
        // Prepare any bindings which should be attached to this device
        val device = _inputManager.getInputDevice(newDeviceId)
        val mappings = _deviceMappings[device.descriptor] ?: return

        _deviceBindings[newDeviceId] = DeviceBindings(mappings)
    }

    override fun onInputDeviceRemoved(oldDeviceId: Int) {
        // Remove any bindings which were attached to this device
        _deviceBindings.remove(oldDeviceId)
    }

    override fun onInputDeviceChanged(deviceId: Int) {
        // no-op
    }
}