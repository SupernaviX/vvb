package com.simongellis.vvb

import android.annotation.SuppressLint
import android.app.Application
import android.content.Context
import android.content.SharedPreferences
import android.net.Uri
import android.widget.Toast
import androidx.preference.PreferenceManager
import com.getkeepsafe.relinker.ReLinker
import com.simongellis.vvb.data.*
import com.simongellis.vvb.emulator.Input
import com.simongellis.vvb.game.GamePakLoader
import org.acra.ACRA
import org.acra.config.httpSender
import org.acra.config.toast
import org.acra.data.StringFormat
import org.acra.ktx.initAcra
import java.io.File
import java.util.*

class VvbApplication : Application() {
    override fun attachBaseContext(base: Context?) {
        super.attachBaseContext(base)

        AcraConfig.load()?.also { config ->
            initAcra {
                buildConfigClass = BuildConfig::class.java
                reportFormat = StringFormat.JSON
                httpSender {
                    uri = config.uri
                    basicAuthLogin = config.login
                    basicAuthPassword = config.password
                }
                toast {
                    text = getString(R.string.application_crashed)
                    @SuppressLint("Range")
                    length = Toast.LENGTH_LONG
                }
            }
        }
    }

    private class AcraConfig(val uri: String, val login: String, val password: String) {
        companion object {
            fun load(): AcraConfig? {
                if (!BuildConfig.ACRA_ENABLED) {
                    return null
                }
                val uri = BuildConfig.ACRA_URI
                val login = BuildConfig.ACRA_BASIC_AUTH_LOGIN ?: return null
                val password = BuildConfig.ACRA_BASIC_AUTH_PASSWORD ?: return null
                return AcraConfig(uri, login, password)
            }
        }
    }

    private val migrations = listOf(
        ::updateMappingSchema,
        ::moveControllersToJson,
        ::moveGamesToJson,
        ::addStateFieldsToGames,
        ::useHashAsGameDataId,
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
        val controllerId = UUID.randomUUID().toString()
        editor.putStringSet("controllers", setOf("$controllerId::Controller 1"))

        for (input in mappedInputs) {
            // Add the mapping to the new controller in the new format
            prefs.getString(input.prefName, null)?.also { savedMapping ->
                val (device, keyCode) = savedMapping.split("::")
                val mappingKey = "controller_${controllerId}_${input.prefName}"
                val mappingValue = "$device::key::$keyCode"
                editor.putStringSet(mappingKey, setOf(mappingValue))
            }

            // and remove the old-format pref
            editor.remove(input.prefName)
        }
    }

    /**
     * Originally, controller key bindings were stored as ::-delimited strings in the main prefs file.
     * Switch to storing them as JSON in a dedicated file, to majorly clean up the code.
     */
    private fun moveControllersToJson(prefs: SharedPreferences, editor: SharedPreferences.Editor) {
        if (!prefs.contains("controllers")) {
            return
        }
        val rawControllers = prefs.getStringSet("controllers", setOf())!!
        val dao = PreferencesDao.forClass<ControllerData>(applicationContext)
        rawControllers.forEach { raw ->
            val (id, name) = raw.split("::", limit = 2)
            val keyMappings = ArrayList<KeyMapping>()
            val axisMappings = ArrayList<AxisMapping>()
            Input.values().forEach { input ->
                val mappingKey = "controller_${id}_${input.prefName}"
                val rawMappings = prefs.getStringSet(mappingKey, setOf())!!
                rawMappings.forEach {
                    val (device, type, data) = it.split("::")
                    if (type == "key") {
                        val keyCode = data.toInt()
                        keyMappings.add(KeyMapping(device, input, keyCode))
                    }
                    if (type == "axis") {
                        val (rawAxis, sign) = data.split('_')
                        val axis = rawAxis.toInt()
                        val isNegative = sign == "-"
                        axisMappings.add(AxisMapping(device, input, axis, isNegative))
                    }
                }
                editor.remove(mappingKey)
            }
            val controller = ControllerData(id, name, keyMappings, axisMappings)
            dao.put(controller)
        }
        editor.remove("controllers")
    }

    /**
     * Originally, the game list was stored as ::-delimited strings in a "recent_games" preference.
     * Switch to storing them as JSON in a dedicated file, to make it more extensible.
     */
    private fun moveGamesToJson(prefs: SharedPreferences, editor: SharedPreferences.Editor) {
        if (!prefs.contains("recent_games")) {
            return
        }
        fun computeOriginalFormatId(uri: Uri) = uri.lastPathSegment!!
            .substringAfterLast('/')
            .substringBeforeLast('.')

        val rawRecentGames = prefs.getStringSet("recent_games", setOf())!!
        val dao = PreferencesDao.forClass<GameData>(applicationContext)
        rawRecentGames.forEach {
            val (rawLastPlayed, rawUri) = it.split("::")
            val lastPlayed = Date(rawLastPlayed.toLong())
            val uri = Uri.parse(rawUri)
            val id = computeOriginalFormatId(uri)
            val game = GameData(id, uri, lastPlayed, 0, true)
            dao.put(game)
        }
        editor.remove("recent_games")
    }

    /**
     * Add an "active state slot" index and "auto save enabled" flag to any games missing them
     */
    @Suppress("UNUSED_PARAMETER")
    private fun addStateFieldsToGames(prefs: SharedPreferences, editor: SharedPreferences.Editor) {
        val dao = PreferencesDao.forClass<GameData>(applicationContext)
        dao.migrateValues {
            it.put("stateSlot", 0)
            it.put("autoSaveEnabled", true)
        }
    }

    /**
     * Originally, this app used a ROM's filename (sans file extension) as an "id" when saving data.
     * Switch to using the ROM's MD5 hash, for more robust game identification.
     */
    @Suppress("UNUSED_PARAMETER")
    private fun useHashAsGameDataId(prefs: SharedPreferences, editor: SharedPreferences.Editor) {
        val dao = PreferencesDao.forClass<GameData>(applicationContext)
        val gamePakLoader = GamePakLoader(applicationContext)

        data class GameData(val oldId: String, val hash: String?, val lastPlayed: Date)

        val allGameData = dao.mapRaw { oldId, raw ->
            val uri = Uri.parse(raw.getString("uri"))
            val hash = gamePakLoader.tryLoad(uri).map { it.hash }.getOrNull()
            val lastPlayed = Date(raw.getLong("lastPlayed"))
            GameData(oldId, hash, lastPlayed)
        }

        // Changing the id means we have to start enforcing uniqueness where we didn't before.
        // For each hash, pick the game data which was most recently played; that's the one to keep.
        val idsToMove = allGameData.groupBy { it.hash }
            .filter { it.key != null }
            .values
            .mapNotNull { records -> records.maxByOrNull { it.lastPlayed } }
            .associateBy { it.oldId }

        val gameDataDir = File(applicationContext.filesDir, "game_data")
        gameDataDir.mkdir()

        allGameData.forEach {
            val oldSram = File(applicationContext.filesDir, "${it.oldId}.srm")
            val oldSaveStates = File(applicationContext.filesDir, "${it.oldId}/save_states")
            if (idsToMove.containsKey(it.oldId)) {
                // copy the save data over to the new location
                val romParent = File(gameDataDir, it.hash!!)
                romParent.mkdir()
                oldSram.renameTo(romParent.resolve(".srm"))
                oldSaveStates.renameTo(romParent.resolve("save_states"))
            } else {
                // throw this save data out, it's unreachable
                oldSram.delete()
            }
            File(applicationContext.filesDir, it.oldId).deleteRecursively()
        }

        dao.migrate { oldId, value ->
            val dataToMove = idsToMove[oldId]
            if (dataToMove != null) {
                // keep this record, and store the hash as the record's new ID
                value.put("id", dataToMove.hash)
                dataToMove.hash // hash is now the id in our database as well
            } else {
                // delete the record, we don't need it anymore
                null
            }
        }
    }
}