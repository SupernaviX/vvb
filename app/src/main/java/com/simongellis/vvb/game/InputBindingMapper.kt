package com.simongellis.vvb.game

import android.content.Context
import android.hardware.input.InputManager
import android.view.KeyEvent
import android.view.MotionEvent
import androidx.core.content.ContextCompat.getSystemService
import com.simongellis.vvb.data.AxisMapping
import com.simongellis.vvb.data.ControllerRepository
import com.simongellis.vvb.data.KeyMapping
import com.simongellis.vvb.data.Mapping
import com.simongellis.vvb.emulator.Input
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import kotlinx.coroutines.suspendCancellableCoroutine
import kotlin.collections.HashMap

class InputBindingMapper(scope: CoroutineScope, context: Context): InputManager.InputDeviceListener {

    data class AxisBinding(val axis: Int, val isNegative: Boolean, val input: Input)

    class DeviceBindings(mappings: List<Mapping>) {
        val keyBindings = mappings
            .filterIsInstance<KeyMapping>()
            .map { it.keyCode to it.input }
            .toMap()
        val axisBindings = mappings
            .filterIsInstance<AxisMapping>()
            .map { AxisBinding(it.axis, it.isNegative, it.input) }
            .groupBy { it.input }
    }

    private val _deviceMappings = ControllerRepository(context)
        .getAllMappings()
        .groupBy { it.device }

    private val _deviceBindings = HashMap<Int, DeviceBindings>()
    private val _inputManager = getSystemService(context, InputManager::class.java)!!

    init {
        val listener = this
        scope.launch {
            _inputManager.registerInputDeviceListener(listener, null)
            _inputManager.inputDeviceIds.forEach {
                onInputDeviceAdded(it)
            }
            suspendCancellableCoroutine {
                it.invokeOnCancellation {
                    _inputManager.unregisterInputDeviceListener(listener)
                }
            }
        }
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

        _deviceBindings[event.deviceId]?.axisBindings?.also { inputBindings ->
            for ((input, bindings) in inputBindings) {
                val active = bindings.any {
                    val axisValue = event.getAxisValue(it.axis)
                    if (it.isNegative) { axisValue < -0.45 } else { axisValue > 0.45 }
                }
                if (active) {
                    pressed.add(input)
                } else {
                    released.add(input)
                }
            }
        }

        return pressed to released
    }

    override fun onInputDeviceAdded(newDeviceId: Int) {
        // Prepare any bindings which should be attached to this device
        val device = _inputManager.getInputDevice(newDeviceId) ?: return
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