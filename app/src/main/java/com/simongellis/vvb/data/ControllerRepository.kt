package com.simongellis.vvb.data

import android.content.Context
import kotlinx.coroutines.flow.map
import java.util.*

class ControllerRepository(context: Context) {
    private val _dao = PreferencesDao.forClass<ControllerData>(context)

    val controllers by lazy {
        _dao.watchAll().map { it.map(::fromData) }
    }

    fun getLiveController(id: String) = _dao.watch(id).map { fromData(it) }

    fun addController(name: String): Controller {
        val controller = Controller(
            UUID.randomUUID().toString(),
            name,
            listOf(),
        )
        _dao.put(toData(controller))
        return controller
    }

    fun putController(controller: Controller) {
        _dao.put(toData(controller))
    }

    fun deleteController(controller: Controller) {
        _dao.delete(controller.id)
    }

    fun putMapping(controllerId: String, mapping: Mapping) {
        val controller = fromData(_dao.get(controllerId) ?: return)
        val newController = controller.copy(
            mappings = controller.mappings.filter { it.input != mapping.input } + mapping
        )
        _dao.put(toData(newController))
    }

    fun addMapping(controllerId: String, mapping: Mapping) {
        val controller = fromData(_dao.get(controllerId) ?: return)
        val newController = controller.copy(
            mappings = controller.mappings.filter { it != mapping } + mapping
        )
        _dao.put(toData(newController))
    }

    fun getAllMappings(): List<Mapping> {
        return _dao.getAll()
            .flatMap { it.keyMappings + it.axisMappings }
    }

    private fun fromData(data: ControllerData): Controller {
        return Controller(data.id, data.name, data.keyMappings + data.axisMappings)
    }

    private fun toData(controller: Controller): ControllerData {
        return ControllerData(
            controller.id,
            controller.name,
            controller.mappings.filterIsInstance<KeyMapping>(),
            controller.mappings.filterIsInstance<AxisMapping>()
        )
    }
}