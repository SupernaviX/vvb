package com.simongellis.vvb.data

import android.content.Context
import kotlinx.coroutines.flow.map
import java.util.*

class ControllerRepository(context: Context) {
    private val dao = PreferencesDao.forClass(ControllerData.serializer(), context)

    val controllers by lazy {
        dao.watchAll().map { it.map(::fromData) }
    }

    fun getLiveController(id: String) = dao.watch(id).map { fromData(it) }

    fun addController(name: String): Controller {
        val controller = Controller(
            UUID.randomUUID().toString(),
            name,
            listOf(),
        )
        dao.put(toData(controller))
        return controller
    }

    fun putController(controller: Controller) {
        dao.put(toData(controller))
    }

    fun deleteController(controller: Controller) {
        dao.delete(controller.id)
    }

    fun putMapping(controllerId: String, mapping: Mapping) {
        val controller = fromData(dao.get(controllerId) ?: return)
        val newController = controller.copy(
            mappings = controller.mappings.filter { it.input != mapping.input } + mapping
        )
        dao.put(toData(newController))
    }

    fun addMapping(controllerId: String, mapping: Mapping) {
        val controller = fromData(dao.get(controllerId) ?: return)
        val newController = controller.copy(
            mappings = controller.mappings.filter { it != mapping } + mapping
        )
        dao.put(toData(newController))
    }

    fun getAllMappings(): List<Mapping> {
        return dao.getAll()
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