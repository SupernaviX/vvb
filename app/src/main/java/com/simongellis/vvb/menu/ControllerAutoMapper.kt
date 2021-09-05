package com.simongellis.vvb.menu

import android.view.InputDevice
import android.view.KeyEvent
import android.view.MotionEvent
import com.simongellis.vvb.data.AxisMapping
import com.simongellis.vvb.data.KeyMapping
import com.simongellis.vvb.data.Mapping
import com.simongellis.vvb.emulator.Input

class ControllerAutoMapper {
    data class AutoMapResult(
        val name: String,
        val mappings: List<Mapping>,
        val fullyMapped: Boolean,
    )

    fun isMappable(device: InputDevice): Boolean {
        if (device.isVirtual) {
            return false
        }
        return listOf(
            InputDevice.SOURCE_DPAD,
            InputDevice.SOURCE_JOYSTICK,
            InputDevice.SOURCE_GAMEPAD
        )
            .any { device.supportsSource(it) }
    }

    fun computeMappings(device: InputDevice): AutoMapResult {
        val mappings = mutableListOf<Mapping>()
        fun tryMapKeys(vararg inputToKeyCode: Pair<Int, Input>) {
            val supportedKeys = device.hasKeys(*inputToKeyCode.map { it.first }.toIntArray())
            inputToKeyCode
                .filterIndexed { index, _ -> supportedKeys[index] }
                .map { (keyCode, input) ->
                    KeyMapping(
                        device.descriptor,
                        input,
                        keyCode
                    )
                }
                .toCollection(mappings)
        }

        fun tryMapAxes(vararg axisToKeyCodes: Pair<Int, Pair<Input?, Input?>>) {
            val ranges = device.motionRanges.map { it.axis }.toSet()
            axisToKeyCodes
                .filter { (axis, _) -> ranges.contains(axis) }
                .forEach { (axis, inputs) ->
                    val (neg, pos) = inputs
                    neg?.also {
                        mappings.add(AxisMapping(device.descriptor, it, axis, true))
                    }
                    pos?.also {
                        mappings.add(AxisMapping(device.descriptor, it, axis, false))
                    }
                }
        }

        tryMapKeys(
            // d-pad
            KeyEvent.KEYCODE_DPAD_LEFT to Input.LL,
            KeyEvent.KEYCODE_DPAD_RIGHT to Input.LR,
            KeyEvent.KEYCODE_DPAD_UP to Input.LU,
            KeyEvent.KEYCODE_DPAD_DOWN to Input.LD,
            KeyEvent.KEYCODE_DPAD_CENTER to Input.A,

            // gamepad
            KeyEvent.KEYCODE_BUTTON_A to Input.A,
            KeyEvent.KEYCODE_BUTTON_B to Input.B,
            KeyEvent.KEYCODE_BUTTON_START to Input.START,
            KeyEvent.KEYCODE_BUTTON_SELECT to Input.SELECT,
            KeyEvent.KEYCODE_BUTTON_L1 to Input.LT,
            KeyEvent.KEYCODE_BUTTON_L2 to Input.LT,
            KeyEvent.KEYCODE_BUTTON_R1 to Input.RT,
            KeyEvent.KEYCODE_BUTTON_R2 to Input.RT,
        )

        tryMapAxes(
            // joysticks
            MotionEvent.AXIS_X to Pair(Input.LL, Input.LR),
            MotionEvent.AXIS_Y to Pair(Input.LU, Input.LD),
            MotionEvent.AXIS_Z to Pair(Input.RL, Input.RR),
            MotionEvent.AXIS_RZ to Pair(Input.RU, Input.RD),

            // d-pad
            MotionEvent.AXIS_HAT_X to Pair(Input.LL, Input.LR),
            MotionEvent.AXIS_HAT_Y to Pair(Input.LU, Input.LD),

            // triggers
            MotionEvent.AXIS_LTRIGGER to Pair(null, Input.LT),
            MotionEvent.AXIS_RTRIGGER to Pair(null, Input.RT),
        )

        val mappedInputs = mappings.map { it.input }.toSet()
        val fullyMapped = Input.values().all { mappedInputs.contains(it) }

        return AutoMapResult(device.name, mappings, fullyMapped)
    }
}