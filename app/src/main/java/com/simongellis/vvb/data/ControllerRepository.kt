package com.simongellis.vvb.data

import android.content.Context
import java.util.*

class ControllerRepository(context: Context) {
    private val dao = PreferencesDao.forClass(Controller.serializer(), context)

    val controllers by lazy {
        dao.watchAll()
    }

    fun getLiveController(id: String) = dao.watch(id)

    fun addController(name: String): Controller {
        val controller = Controller(
            UUID.randomUUID().toString(),
            name,
            listOf(),
            listOf()
        )
        dao.put(controller)
        return controller
    }

    fun putController(controller: Controller) {
        dao.put(controller)
    }

    fun deleteController(controller: Controller) {
        dao.delete(controller.id)
    }

    fun putMapping(controllerId: String, mapping: Mapping) {
        val controller = dao.get(controllerId) ?: return
        val newController = updateMappings(controller, mapping) { it.input == mapping.input }
        dao.put(newController)
    }

    fun addMapping(controllerId: String, mapping: Mapping) {
        val controller = dao.get(controllerId) ?: return
        val newController = updateMappings(controller, mapping) { it == mapping }
        dao.put(newController)
    }

    fun getAllMappings(): List<Mapping> {
        return dao.getAll()
            .flatMap { it.keyMappings + it.axisMappings }
    }

    private fun updateMappings(controller: Controller, mapping: Mapping, replaceWhen: (Mapping) -> Boolean): Controller {
        return controller.copy(
            keyMappings = maybeAddMapping(controller.keyMappings.filter{ !replaceWhen(it) }, mapping),
            axisMappings = maybeAddMapping(controller.axisMappings.filter{ !replaceWhen(it) }, mapping)
        )
    }

    private inline fun <reified T: Mapping> maybeAddMapping(mappings: List<T>, mapping: Mapping): List<T> {
        if (mapping is T && !mappings.contains(mapping)) {
            return mappings + mapping
        }
        return mappings
    }
}