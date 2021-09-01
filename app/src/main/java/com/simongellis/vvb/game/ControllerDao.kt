package com.simongellis.vvb.game

import android.content.Context
import android.content.SharedPreferences
import android.view.KeyEvent
import android.view.MotionEvent
import androidx.core.content.edit
import androidx.preference.PreferenceManager
import com.fredporciuncula.flow.preferences.FlowSharedPreferences
import com.simongellis.vvb.emulator.Input
import com.simongellis.vvb.utils.asStateFlow
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.map
import java.util.*
import kotlin.collections.HashSet

class ControllerDao(scope: CoroutineScope, preferences: SharedPreferences) {
    constructor(scope: CoroutineScope, context: Context): this(scope, PreferenceManager.getDefaultSharedPreferences(context))
    private val _preferences = FlowSharedPreferences(preferences)
    private val _rawControllers = _preferences.getStringSet("controllers", setOf())

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

    val controllers by lazy {
        _rawControllers.asStateFlow(scope) { parseControllers(it) }
    }

    fun getLiveController(id: String)
        = controllers.map { c -> c.first { it.id == id } }

    fun addController(name: String): Controller {
        val controllers = _rawControllers.get()
        val controller = Controller(UUID.randomUUID().toString(), name)
        val newControllers = HashSet(controllers).apply { add(controller.toString()) }
        _rawControllers.set(newControllers)
        return controller
    }

    fun putController(controller: Controller) {
        val newControllers = getControllers()
            .filter { it.id != controller.id }
            .plus(controller)
            .map { it.toString() }
            .toSet()
        _rawControllers.set(newControllers)
    }

    fun deleteController(controller: Controller) {
        val controllers = _rawControllers.get()
        val newControllers = HashSet(controllers).apply { remove(controller.toString()) }
        _rawControllers.set(newControllers)
        _preferences.sharedPreferences.edit {
            Input.values()
                .map { getMappingKey(controller.id, it) }
                .forEach { remove(it) }
        }
    }

    fun putMapping(controllerId: String, mapping: Mapping) {
        val raw = getRawMapping(controllerId, mapping.input)
        val value = setOf(mapping.control)
        raw.set(value)
    }
    fun addMapping(controllerId: String, mapping: Mapping) {
        val raw = getRawMapping(controllerId, mapping.input)
        val value = HashSet(raw.get()).apply { add(mapping.control) }
        raw.set(value)
    }

    fun getLiveMappings(controllerId: String, input: Input) =
        getRawMapping(controllerId, input).asFlow()
            .map { parseMappings(input, it) }

    fun getAllMappings(): List<Mapping> {
        return getControllers().flatMap { getMappings(it.id) }
    }
    private fun getMappings(controllerId: String): List<Mapping> {
        return Input.values().flatMap {
            val raw = getRawMapping(controllerId, it)
            parseMappings(it, raw.get())
        }
    }

    private fun getControllers(): List<Controller> {
        return parseControllers(_rawControllers.get())
    }
    private fun parseControllers(raw: Set<String>): List<Controller> {
        return raw.map { Controller.fromString(it) }
    }

    private fun getRawMapping(controllerId: String, input: Input)
        = _preferences.getStringSet(getMappingKey(controllerId, input))

    private fun parseMappings(input: Input, rawMappings: Set<String>): List<Mapping> {
        return rawMappings.mapNotNull {
            val (device, type, data) = it.split("::")
            when (type) {
                "key" -> KeyMapping.parse(device, input, data)
                "axis" -> AxisMapping.parse(device, input, data)
                else -> null
            }
        }
    }

    private fun getMappingKey(controllerId: String, input: Input): String {
        return "controller_${controllerId}_${input.prefName}"
    }
}