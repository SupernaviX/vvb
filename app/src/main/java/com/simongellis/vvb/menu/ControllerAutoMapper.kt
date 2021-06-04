package com.simongellis.vvb.menu

import android.view.InputDevice
import com.simongellis.vvb.game.ControllerDao

class ControllerAutoMapper {
    data class AutoMapResult(
        val name: String,
        val mappings: List<ControllerDao.Mapping>
    )

    fun isMappable(device: InputDevice): Boolean {
        if (device.isVirtual) {
            return false
        }
        return listOf(InputDevice.SOURCE_DPAD, InputDevice.SOURCE_JOYSTICK, InputDevice.SOURCE_GAMEPAD)
            .any { device.supportsSource(it) }
    }

    fun computeMappings(device: InputDevice): AutoMapResult {
        return AutoMapResult(device.name, listOf())
    }
}