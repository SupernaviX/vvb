package com.simongellis.vvb

import android.app.Application
import android.content.SharedPreferences
import androidx.preference.PreferenceManager
import com.simongellis.vvb.emulator.Input
import java.util.*

class VvbApplication: Application() {
    private val migrations = listOf(
        ::updateControllerSchema
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

    private fun updateControllerSchema(prefs: SharedPreferences, editor: SharedPreferences.Editor) {
        val boundInputs = Input.values()
            .map { it.prefName }
            .filter { it != null && prefs.contains(it) }
        if (boundInputs.isEmpty()) {
            return
        }
        val controllerId = UUID.randomUUID().toString()
        val controllerName = "Controller 1"
        editor.putStringSet("controllers", setOf("$controllerId::$controllerName"))
        for (input in boundInputs) {
            val savedBinding = prefs.getString(input, null)!!
            val (device, keyCode) = savedBinding.split("::")
            editor.putString("controller_${controllerId}_$input", "button::${device}_$keyCode")
        }
    }
}