package com.simongellis.vvb.game

import android.content.Context
import android.content.SharedPreferences
import androidx.preference.PreferenceManager
import com.simongellis.vvb.emulator.Input

class ControllerPreferences(context: Context) {
    data class Mapping(
        val input: Input,
        val device: String,
        val keyCode: Int)

    private val controllerIds: List<String>

    val mappings: List<Mapping>
    val deviceMappings: Map<String, List<Mapping>>

    init {
        val prefs = PreferenceManager.getDefaultSharedPreferences(context)
        val controllerDescriptors = prefs.getStringSet("controllers", setOf())!!
        controllerIds = controllerDescriptors.map { it.substringBefore("::") }

        mappings = Input.values().flatMap { input -> getMappings(input, prefs) }
        deviceMappings = mappings.groupBy { it.device }
    }

    private fun getMappings(input: Input, prefs: SharedPreferences): List<Mapping> {
        if (input.prefName == null) {
            return listOf()
        }
        return controllerIds.mapNotNull { id ->
            val pref = "controller_${id}_${input.prefName}"
            val rawMapping = prefs.getString(pref, null)
            rawMapping?.let {
                val (_, data) = it.split("::")
                val (device, keyCode) = data.split("_")
                Mapping(input, device, keyCode.toInt(10))
            }
        }
    }
}