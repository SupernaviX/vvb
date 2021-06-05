package com.simongellis.vvb.game

import android.content.Context
import android.content.SharedPreferences
import android.view.KeyEvent
import android.view.MotionEvent
import androidx.core.content.edit
import androidx.preference.PreferenceManager
import com.simongellis.vvb.emulator.Input
import java.util.*
import kotlin.collections.HashSet

class ControllerDao(private val preferences: SharedPreferences) {
    constructor(context: Context): this(PreferenceManager.getDefaultSharedPreferences(context))

    data class Controller(val id: String, val name: String) {
        override fun toString(): String {
            return "$id::$name"
        }
        companion object {
            fun fromString(value: String): Controller {
                val (id, name) = value.split("::", limit = 2)
                return Controller(id, name)
            }
        }
    }

    interface Mapping {
        val device: String
        val input: Input
        val control: String
    }
    data class KeyMapping(
        override val device: String,
        override val input: Input,
        val keyCode: Int,
    ): Mapping {
        override fun toString(): String {
            return KeyEvent.keyCodeToString(keyCode).removePrefix("KEYCODE_")
        }

        override val control: String
            get() = "$device::key::$keyCode"
        companion object {
            fun parse(device: String, input: Input, data: String): KeyMapping {
                return KeyMapping(device, input, data.toInt(10))
            }
        }
    }
    data class AxisMapping(
        override val device: String,
        override val input: Input,
        val axis: Int,
        val isNegative: Boolean,
    ): Mapping {
        private val sign = if (isNegative) { '-' } else { '+' }

        override fun toString(): String {
            return "${MotionEvent.axisToString(axis).removePrefix("AXIS_")} $sign"
        }

        override val control: String
            get() = "$device::axis::${axis}_$sign"
        companion object {
            fun parse(device: String, input: Input, data: String): AxisMapping {
                val (axis, sign) = data.split("_")
                return AxisMapping(device, input, axis.toInt(10), sign == "-")
            }
        }
    }

    fun getControllers(): List<Controller> {
        return readControllers().map { Controller.fromString(it) }
    }

    fun getController(id: String): Controller {
        return getControllers().first { it.id == id }
    }

    fun addController(name: String): Controller {
        val controllers = readControllers()
        val controller = Controller(UUID.randomUUID().toString(), name)
        val newControllers = HashSet(controllers).apply { add(controller.toString()) }
        preferences.edit {
            putStringSet("controllers", newControllers)
        }
        return controller
    }

    fun putController(controller: Controller) {
        val newControllers = getControllers()
            .filter { it.id != controller.id }
            .plus(controller)
            .map { it.toString() }
            .toSet()
        preferences.edit {
            putStringSet("controllers", newControllers)
        }
    }

    fun deleteController(controller: Controller) {
        val controllers = readControllers()
        val newControllers = HashSet(controllers).apply { remove(controller.toString()) }
        preferences.edit {
            putStringSet("controllers", newControllers)
            Input.values()
                .map { getMappingKey(controller.id, it) }
                .forEach { remove(it) }
        }
    }

    fun putMapping(controllerId: String, mapping: Mapping) {
        val key = getMappingKey(controllerId, mapping.input)
        preferences.edit {
            putStringSet(key, setOf(mapping.control))
        }
    }
    fun addMapping(controllerId: String, mapping: Mapping) {
        val key = getMappingKey(controllerId, mapping.input)
        val controls = preferences.getStringSet(key, setOf())!!
        preferences.edit {
            putStringSet(key, HashSet(controls).apply { add(mapping.control) })
        }
    }

    fun getAllMappings(): List<Mapping> {
        return getControllers().flatMap { getMappings(it.id) }
    }
    private fun getMappings(controllerId: String): List<Mapping> {
        return Input.values().flatMap { getMappings(controllerId, it) }
    }
    fun getMappings(controllerId: String, input: Input): List<Mapping> {
        val key = getMappingKey(controllerId, input)
        val rawMappings = preferences.getStringSet(key, setOf())!!
        return rawMappings.mapNotNull {
            val (device, type, data) = it.split("::")
            when (type) {
                "key" -> KeyMapping.parse(device, input, data)
                "axis" -> AxisMapping.parse(device, input, data)
                else -> null
            }
        }
    }

    private fun readControllers(): Set<String> {
        return preferences.getStringSet("controllers", setOf())!!
    }
    private fun getMappingKey(controllerId: String, input: Input): String {
        return "controller_${controllerId}_${input.prefName}"
    }
}