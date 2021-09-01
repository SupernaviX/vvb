package com.simongellis.vvb.menu

import android.app.Application
import android.view.InputDevice
import androidx.annotation.StringRes
import androidx.lifecycle.*
import com.simongellis.vvb.R
import com.simongellis.vvb.game.ControllerDao
import com.simongellis.vvb.utils.mapAsState
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.launch

class ControllersViewModel(application: Application): AndroidViewModel(application) {
    private val _controllerDao = ControllerDao(viewModelScope, application)
    private val _autoMapper = ControllerAutoMapper()

    private enum class State { Normal, Renaming, Deleting }
    private val _state = MutableStateFlow(State.Normal)

    private var _renamingController: ControllerDao.Controller? = null

    val controllers by _controllerDao::controllers
    private val _editingController = MutableSharedFlow<ControllerDao.Controller>()
    val editingController = _editingController.asSharedFlow()

    val renameLabel = _state.mapAsState(viewModelScope) {
        if (it == State.Renaming) {
            R.string.controller_menu_choose_rename
        } else {
            R.string.controller_menu_rename_controller
        }
    }
    val deleteLabel = _state.mapAsState(viewModelScope) {
        if (it == State.Deleting) {
            R.string.controller_menu_choose_delete
        } else {
            R.string.controller_menu_delete_controller
        }
    }

    class NameDialog(@StringRes val action: Int, val initialValue: String)
    private val _showNameDialog = MutableSharedFlow<NameDialog>()
    val showNameDialog = _showNameDialog.asSharedFlow()
    private val _showAutoMapDialog = MutableSharedFlow<Unit>()
    val showAutoMapDialog = _showAutoMapDialog.asSharedFlow()

    fun promptAutoMap() {
        viewModelScope.launch {
            _showAutoMapDialog.emit(Unit)
            _state.value = State.Normal
        }
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
        viewModelScope.launch {
            _editingController.emit(controller)
        }
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
        viewModelScope.launch {
            _showNameDialog.emit(NameDialog(action, value))
        }
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