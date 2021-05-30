package com.simongellis.vvb.game

import android.content.Context
import android.content.SharedPreferences
import androidx.preference.PreferenceManager
import com.simongellis.vvb.emulator.Input

class ControllerPreferences(context: Context) {
    interface Mapping { val device: String }
    data class KeyMapping(
        override val device: String,
        val keyCode: Int,
        val input: Input,
    ): Mapping
    data class AxisMapping(
        override val device: String,
        val axis: Int,
        val isNegative: Boolean,
        val input: Input,
    ): Mapping

    private val controllerIds: List<String>

    val deviceMappings: Map<String, List<Mapping>>

    init {
        val prefs = PreferenceManager.getDefaultSharedPreferences(context)
        val controllerDescriptors = prefs.getStringSet("controllers", setOf())!!
        controllerIds = controllerDescriptors.map { it.substringBefore("::") }

        val mappings = Input.values().flatMap { input -> getMappings(input, prefs) }
        deviceMappings = mappings.groupBy { it.device }
    }

    private fun getMappings(input: Input, prefs: SharedPreferences): List<Mapping> {
        if (input.prefName == null) {
            return listOf()
        }
        return controllerIds.mapNotNull { id ->
            val pref = "controller_${id}_${input.prefName}"
            val rawMapping = prefs.getString(pref, null)
            rawMapping?.let { parseMapping(it, input) }
        }
    }

    private fun parseMapping(rawMapping: String, input: Input): Mapping? {
        val (device, type, data) = rawMapping.split("::")
        return when (type) {
            "key" -> KeyMapping(device, data.toInt(10), input)
            "axis" -> {
                val (axis, sign) = data.split("_")
                AxisMapping(device, axis.toInt(10), sign == "-", input)
            }
            else -> null
        }
    }
}