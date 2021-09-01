package com.simongellis.vvb.menu

import android.app.Application
import android.view.InputDevice
import androidx.annotation.StringRes
import androidx.lifecycle.*
import com.simongellis.vvb.R
import com.simongellis.vvb.game.ControllerDao
import com.simongellis.vvb.utils.LiveEvent

class ControllersViewModel(application: Application): AndroidViewModel(application) {
    private val _controllerDao = ControllerDao(viewModelScope, application)
    private val _autoMapper = ControllerAutoMapper()

    private enum class State { Normal, Renaming, Deleting }
    private val _state = MutableLiveData(State.Normal)

    private var _renamingController: ControllerDao.Controller? = null

    val controllers by _controllerDao::controllers
    val editingController = LiveEvent<ControllerDao.Controller>()

    val renameLabel = Transformations.map(_state) {
        if (it == State.Renaming) {
            R.string.controller_menu_choose_rename
        } else {
            R.string.controller_menu_rename_controller
        }
    }
    val deleteLabel = Transformations.map(_state) {
        if (it == State.Deleting) {
            R.string.controller_menu_choose_delete
        } else {
            R.string.controller_menu_delete_controller
        }
    }

    class NameDialog(@StringRes val action: Int, val initialValue: String)
    val showNameDialog = LiveEvent<NameDialog>()
    val showAutoMapDialog = LiveEvent<Unit>()

    fun promptAutoMap() {
        _state.value = State.Normal
        showAutoMapDialog.emit(Unit)
    }
    fun isMappable(device: InputDevice): Boolean = _autoMapper.isMappable(device)

    fun promptAddController() {
        _state.value = State.Normal
        promptForName(
            R.string.controller_menu_create,
            "Controller ${controllers.value.size + 1}"
        )
    }

    fun toggleRenaming() = toggleState(State.Renaming)

    fun toggleDeleting() = toggleState(State.Deleting)

    fun doAction(controller: ControllerDao.Controller) {
        when (_state.value) {
            State.Normal -> editControllerMappings(controller)
            State.Renaming -> promptRenameController(controller)
            State.Deleting -> deleteController(controller)
        }
    }

    fun chooseControllerName(name: String) {
        val renamingController = _renamingController
        _renamingController = null

        if (renamingController == null) {
            val controller = _controllerDao.addController(name)
            editControllerMappings(controller)
        } else {
            val controller = renamingController.copy(name = name)
            _controllerDao.putController(controller)
        }
    }

    fun performAutoMap(device: InputDevice) {
        val result = _autoMapper.computeMappings(device)
        val controller = _controllerDao.addController(result.name)
        for (mapping in result.mappings) {
            _controllerDao.addMapping(controller.id, mapping)
        }
        if (!result.fullyMapped) {
            editControllerMappings(controller)
        }
    }

    private fun editControllerMappings(controller: ControllerDao.Controller) {
        editingController.emit(controller)
    }

    private fun promptRenameController(controller: ControllerDao.Controller) {
        _state.value = State.Normal
        _renamingController = controller
        promptForName(
            R.string.controller_menu_rename,
            controller.name
        )
    }

    private fun promptForName(@StringRes action: Int, value: String) {
        showNameDialog.emit(NameDialog(action, value))
    }

    private fun deleteController(controller: ControllerDao.Controller) {
        _state.value = State.Normal
        _controllerDao.deleteController(controller)
    }

    private fun toggleState(newState: State) {
        val oldState = _state.value
        _state.value = if (oldState == newState) {
            State.Normal
        } else {
            newState
        }
    }
}