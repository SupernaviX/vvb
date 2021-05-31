package com.simongellis.vvb.game

import android.content.Context
import android.content.SharedPreferences
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
    }
    data class KeyMapping(
        override val device: String,
        override val input: Input,
        val keyCode: Int,
    ): Mapping {
        override fun toString(): String {
            return "$device::key::$keyCode"
        }
    }
    data class AxisMapping(
        override val device: String,
        override val input: Input,
        val axis: Int,
        val isNegative: Boolean,
    ): Mapping {
        override fun toString(): String {
            val sign = if (isNegative) { '-' } else { '+' }
            return "$device::axis::${axis}_$sign"
        }
    }

    fun getControllers(): List<Controller> {
        return readControllers().map { Controller.fromString(it) }
    }

    fun getController(id: String): Controller {
        return getControllers().first { it.id == id }
    }

    fun addController(): Controller {
        val controllers = readControllers()
        val controller = Controller(UUID.randomUUID().toString(), "Controller ${controllers.size + 1}")
        val newControllers = HashSet(controllers).apply { add(controller.toString()) }
        preferences.edit {
            putStringSet("controllers", newControllers)
        }
        return controller
    }

    fun deleteController(controller: Controller) {
        val controllers = readControllers()
        val newControllers = HashSet(controllers).apply { remove(controller.toString()) }
        preferences.edit {
            putStringSet("controllers", newControllers)
            getInputs()
                .map { getMappingKey(controller.id, it) }
                .forEach { remove(it) }
        }
    }

    fun addMapping(controllerId: String, mapping: Mapping) {
        val key = getMappingKey(controllerId, mapping.input)
        preferences.edit {
            putString(key, mapping.toString())
        }
    }

    fun hasMapping(controllerId: String, input: Input): Boolean {
        val key = getMappingKey(controllerId, input)
        return preferences.contains(key)
    }

    fun getAllMappings(): List<Mapping> {
        return getControllers().flatMap { getMappings(it.id) }
    }
    private fun getMappings(controllerId: String): List<Mapping> {
        return getInputs().mapNotNull { getMapping(controllerId, it) }
    }
    private fun getMapping(controllerId: String, input: Input): Mapping? {
        val key = getMappingKey(controllerId, input)
        val rawMapping = preferences.getString(key, null)
            ?: return null
        val (device, type, data) = rawMapping.split("::")
        return when (type) {
            "key" -> KeyMapping(device, input, data.toInt(10))
            "axis" -> {
                val (axis, sign) = data.split("_")
                AxisMapping(device, input, axis.toInt(10), sign == "-")
            }
            else -> null
        }
    }

    private fun readControllers(): Set<String> {
        return preferences.getStringSet("controllers", setOf())!!
    }
    private fun getInputs(): List<Input> {
        return Input.values().filter { it.prefName != null }
    }
    private fun getMappingKey(controllerId: String, input: Input): String {
        return "controller_${controllerId}_${input.prefName}"
    }
}