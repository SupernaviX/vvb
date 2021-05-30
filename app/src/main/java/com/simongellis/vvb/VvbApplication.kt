package com.simongellis.vvb

import android.app.Application
import android.content.SharedPreferences
import androidx.preference.PreferenceManager
import com.simongellis.vvb.emulator.Input
import java.util.*

class VvbApplication: Application() {
    private val migrations = listOf(
        ::updateMappingSchema
    )

    override fun onCreate() {
        super.onCreate()

        val prefs = PreferenceManager.getDefaultSharedPreferences(baseContext)

        var appliedCount = prefs.getInt("_migration_count", 0)
        if (appliedCount == migrations.size) return

        val editor = prefs.edit()

        while (appliedCount < migrations.size) {
            migrations[appliedCount](prefs, editor)
            appliedCount++
            editor.putInt("_migration_count", appliedCount)

            // .apply is async, but this is still safe because
            //  1. the in-memory preference map is updated synchronously
            //  2. later writes "win"
            editor.apply()
        }
    }

    // update the schema used to store input mapping in preferences
    // to support multiple controllers and multiple kinds of mapping
    private fun updateMappingSchema(prefs: SharedPreferences, editor: SharedPreferences.Editor) {
        val mappedInputs = Input.values()
            .map { it.prefName }
            .filter { it != null && prefs.contains(it) }
        if (mappedInputs.isEmpty()) {
            return
        }

        // Define a new controller
        val controllerId = UUID.randomUUID().toString()
        val controllerName = "Controller 1"
        editor.putStringSet("controllers", setOf("$controllerId::$controllerName"))

        for (input in mappedInputs) {
            // Add the mapping to the new controller in the new format
            val savedMapping = prefs.getString(input, null)!!
            val (device, keyCode) = savedMapping.split("::")
            editor.putString("controller_${controllerId}_$input", "$device::button::$keyCode")

            // and remove the old-format pref
            editor.remove(input)
        }
    }
}