package com.simongellis.vvb.data

import android.content.Context
import com.fredporciuncula.flow.preferences.FlowSharedPreferences
import com.fredporciuncula.flow.preferences.Preference
import kotlinx.coroutines.flow.*
import kotlinx.serialization.KSerializer
import kotlinx.serialization.json.Json

class PreferencesDao<T: Entity>(clazz: Class<T>, private val serializer: KSerializer<T>, context: Context) {
    private val _preferences =
        FlowSharedPreferences(context.getSharedPreferences(clazz.simpleName, 0))
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
        getPreference(value.id).set(serialize(value))
        _ids.set(_ids.get() + value.id)
    }

    fun delete(id: String) {
        getPreference(id).delete()
        _ids.set(_ids.get() - id)
        _valueFlows.remove(id)
        _prefs.remove(id)
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
        inline fun <reified T: Entity> forClass(serializer: KSerializer<T>, context: Context) = PreferencesDao(T::class.java, serializer, context)
    }
}