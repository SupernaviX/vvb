package com.simongellis.vvb

import android.annotation.SuppressLint
import android.app.Application
import android.content.Context
import android.content.SharedPreferences
import android.widget.Toast
import androidx.preference.PreferenceManager
import com.getkeepsafe.relinker.ReLinker
import com.simongellis.vvb.emulator.Input
import com.simongellis.vvb.game.ControllerDao
import org.acra.ACRA
import org.acra.config.httpSender
import org.acra.config.toast
import org.acra.data.StringFormat
import org.acra.ktx.initAcra

class VvbApplication: Application() {
    override fun attachBaseContext(base: Context?) {
        super.attachBaseContext(base)

        if (BuildConfig.ACRA_ENABLED) initAcra {
            buildConfigClass = BuildConfig::class.java
            reportFormat = StringFormat.JSON
            httpSender {
                uri = BuildConfig.ACRA_URI
                basicAuthLogin = BuildConfig.ACRA_BASIC_AUTH_LOGIN
                basicAuthPassword = BuildConfig.ACRA_BASIC_AUTH_PASSWORD
            }
            toast {
                text = getString(R.string.application_crashed)
                @SuppressLint("Range")
                length = Toast.LENGTH_LONG
            }
        }
    }

    private val migrations = listOf(
        ::updateMappingSchema
    )

    override fun onCreate() {
        super.onCreate()
        ReLinker.loadLibrary(this, "vvb")

        if (ACRA.isACRASenderServiceProcess()) {
            return
        }

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
            .filter { prefs.contains(it.prefName) }
        if (mappedInputs.isEmpty()) {
            return
        }

        // Define a new controller
        val controllerDao = ControllerDao(prefs)
        val controller = controllerDao.addController("Controller 1")

        for (input in mappedInputs) {
            // Add the mapping to the new controller in the new format
            val savedMapping = prefs.getString(input.prefName, null)!!
            val (device, keyCode) = savedMapping.split("::")
            val mapping = ControllerDao.KeyMapping(device, input, keyCode.toInt(10))

            controllerDao.addMapping(controller.id, mapping)

            // and remove the old-format pref
            editor.remove(input.prefName)
        }
    }

}