package com.simongellis.vvb.data

import android.content.Context
import com.fredporciuncula.flow.preferences.FlowSharedPreferences
import com.fredporciuncula.flow.preferences.Preference
import kotlinx.coroutines.flow.*
import kotlinx.serialization.KSerializer
import kotlinx.serialization.json.Json
import kotlinx.serialization.serializer
import org.json.JSONObject

class PreferencesDao<T: Entity>(className: String, private val serializer: KSerializer<T>, context: Context) {
    private val _preferences =
        FlowSharedPreferences(context.getSharedPreferences(className, 0))
    private val _ids = _preferences.getStringSet("ids")
    private val _prefs = HashMap<String, Preference<String?>>()
    private val _valueFlows = HashMap<String, Flow<String?>>()

    fun getAll(): List<T> {
        return _ids.get()
            .mapNotNull(this::getRaw)
            .map(this::deserialize)
    }

    fun watchAll(): Flow<List<T>> {
        return _ids.asFlow()
            .flatMapLatest { ids ->
                combine(ids.map(this::getRawFlow)) {
                    it.filterNotNull().map(this::deserialize)
                }
            }
    }

    fun get(id: String): T? {
        return getRaw(id)?.let { deserialize(it) }
    }

    fun watch(id: String): Flow<T> {
        return getRawFlow(id).filterNotNull().map { deserialize(it) }
    }

    fun put(value: T) {
        putRaw(value.id, serialize(value))
    }

    private fun putRaw(id: String, raw: String) {
        getPreference(id).set(raw)
        _ids.set(_ids.get() + id)
    }

    fun delete(id: String) {
        getPreference(id).delete()
        _ids.set(_ids.get() - id)
        _valueFlows.remove(id)
        _prefs.remove(id)
    }

    fun <T> mapRaw(transform: (id: String, value: JSONObject) -> T): List<T> {
        return _ids.get()
            .mapNotNull { id -> getRaw(id)?.let { id to JSONObject(it) }}
            .map { (id, value) -> transform(id, value) }
    }

    /**
     * The given function should return the new id to use for the record,
     * or null to delete the record
     */
    fun migrate(transform: (id: String, value: JSONObject) -> String?) {
        val allValues = _ids.get()
            .mapNotNull { id -> getRaw(id)?.let { id to JSONObject(it) } }
        allValues.forEach { (oldId, value) ->
            val newId = transform(oldId, value)
            if (oldId == newId) {
                getPreference(newId).set(value.toString())
            } else {
                delete(oldId)
                if (newId != null) {
                    putRaw(newId, value.toString())
                }
            }
        }
    }

    fun migrateValues(transform: (value: JSONObject) -> Unit) {
        migrate { id, value ->
            transform(value)
            id
        }
    }

    private fun getRaw(id: String): String? {
        return getPreference(id).get()
    }

    private fun getRawFlow(id: String): Flow<String?> {
        return _valueFlows.getOrPut(id) {
            getPreference(id)
                .asFlow()
                .distinctUntilChanged()
        }
    }

    private fun getPreference(id: String): Preference<String?> {
        val key = "entity_$id"
        return _prefs.getOrPut(key) {
            _preferences.getNullableString(key, null)
        }
    }

    private fun serialize(value: T): String {
        return Json.encodeToString(serializer, value)
    }

    private fun deserialize(raw: String): T {
        return Json.decodeFromString(serializer, raw)
    }

    companion object {
        inline fun <reified T : Entity> forClass(context: Context) = PreferencesDao<T>(T::class.java.simpleName, serializer(), context)
    }
}