package com.simongellis.vvb.menu

import android.app.Application
import android.view.InputDevice
import androidx.annotation.StringRes
import androidx.lifecycle.*
import com.simongellis.vvb.R
import com.simongellis.vvb.emulator.Input
import com.simongellis.vvb.game.ControllerDao
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine

class ControllerInputViewModel(application: Application, savedStateHandle: SavedStateHandle): AndroidViewModel(application) {
    private val _controllerDao = ControllerDao(viewModelScope, application)
    private val _id: String = savedStateHandle["id"]!!

    sealed class InputDisplay {
        data class Resource(@StringRes val id: Int): InputDisplay()
        data class Text(val value: String): InputDisplay()
    }
    data class BindingInfo(val input: Input, val multiple: Boolean)
    private val _binding = MutableStateFlow<BindingInfo?>(null)

    val controller = _controllerDao.getLiveController(_id).asLiveData()

    val inputSummaries = Input.values()
        .map { it to getInputSummary(it) }
        .toMap()

    fun startBinding(input: Input, multiple: Boolean) {
        _binding.value = BindingInfo(input, multiple)
    }

    fun persistKeyMapping(device: InputDevice, keyCode: Int): Boolean {
        val (input, multiple) = _binding.value ?: return false
        val mapping = ControllerDao.KeyMapping(device.descriptor, input, keyCode)
        persistMapping(mapping, multiple)
        return true
    }
    fun persistAxisMapping(device: InputDevice, axis: Int, isNegative: Boolean): Boolean {
        val (input, multiple) = _binding.value ?: return false
        val mapping = ControllerDao.AxisMapping(device.descriptor, input, axis, isNegative)
        persistMapping(mapping, multiple)
        return true
    }
    private fun persistMapping(mapping: ControllerDao.Mapping, multiple: Boolean) {
        if (multiple) {
            _controllerDao.addMapping(_id, mapping)
        } else {
            _controllerDao.putMapping(_id, mapping)
        }
        _binding.value = null
    }

    private fun getInputSummary(input: Input): LiveData<InputDisplay> {
        val mappings = _controllerDao.getLiveMappings(_id, input)

        fun getMessage(currBinding: BindingInfo?, currMappings: List<ControllerDao.Mapping>): InputDisplay {
            if (currBinding?.input == input) {
                if (currBinding.multiple) {
                    return InputDisplay.Resource(R.string.input_menu_add_mapping)
                }
                return InputDisplay.Resource(R.string.input_menu_put_mapping)
            }
            if (currMappings.isEmpty()) {
                return InputDisplay.Resource(R.string.input_menu_unmapped)
            }
            return InputDisplay.Text(currMappings.joinToString(", "))
        }

        val summarizer = _binding.combine(mappings) { b, m -> getMessage(b, m) }
        return summarizer.asLiveData()
    }
}