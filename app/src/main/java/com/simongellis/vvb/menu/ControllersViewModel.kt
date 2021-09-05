package com.simongellis.vvb.menu

import android.app.Application
import android.view.InputDevice
import androidx.annotation.StringRes
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import com.simongellis.vvb.R
import com.simongellis.vvb.data.Controller
import com.simongellis.vvb.data.ControllerRepository
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.launch

class ControllersViewModel(application: Application): AndroidViewModel(application) {
    private val _controllerRepo = ControllerRepository(application)
    private val _autoMapper = ControllerAutoMapper()

    private enum class State { Normal, Renaming, Deleting }
    private val _state = MutableStateFlow(State.Normal)

    private var _renamingController: Controller? = null

    val controllers by _controllerRepo::controllers
    private val _editingController = MutableSharedFlow<Controller>()
    val editingController = _editingController.asSharedFlow()

    private val _newControllerName = controllers
        .map { "Controller ${it.size + 1}" }
        .stateIn(viewModelScope, SharingStarted.Eagerly, "Controller 1")

    val renameLabel = _state.map {
        if (it == State.Renaming) {
            R.string.controller_menu_choose_rename
        } else {
            R.string.controller_menu_rename_controller
        }
    }
    val deleteLabel = _state.map {
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
            _newControllerName.value
        )
    }

    fun toggleRenaming() = toggleState(State.Renaming)

    fun toggleDeleting() = toggleState(State.Deleting)

    fun doAction(controller: Controller) {
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
            val controller = _controllerRepo.addController(name)
            editControllerMappings(controller)
        } else {
            val controller = renamingController.copy(name = name)
            _controllerRepo.putController(controller)
        }
    }

    fun performAutoMap(device: InputDevice) {
        val result = _autoMapper.computeMappings(device)
        val controller = _controllerRepo.addController(result.name)
        for (mapping in result.mappings) {
            _controllerRepo.addMapping(controller.id, mapping)
        }
        if (!result.fullyMapped) {
            editControllerMappings(controller)
        }
    }

    private fun editControllerMappings(controller: Controller) {
        viewModelScope.launch {
            _editingController.emit(controller)
        }
    }

    private fun promptRenameController(controller: Controller) {
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

    private fun deleteController(controller: Controller) {
        _state.value = State.Normal
        _controllerRepo.deleteController(controller)
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