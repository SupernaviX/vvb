package com.simongellis.vvb.menu

import android.app.Application
import android.view.InputDevice
import androidx.annotation.StringRes
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.SavedStateHandle
import com.simongellis.vvb.R
import com.simongellis.vvb.data.AxisMapping
import com.simongellis.vvb.data.ControllerRepository
import com.simongellis.vvb.data.KeyMapping
import com.simongellis.vvb.data.Mapping
import com.simongellis.vvb.emulator.Input
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map

class ControllerInputViewModel(application: Application, savedStateHandle: SavedStateHandle) :
    AndroidViewModel(application) {
    private val _controllerRepo = ControllerRepository(application)
    private val _id: String = savedStateHandle["id"]!!

    sealed class InputDisplay {
        data class Resource(@StringRes val id: Int) : InputDisplay()
        data class Text(val value: String) : InputDisplay()
    }

    data class BindingInfo(val input: Input, val multiple: Boolean)

    private val _binding = MutableStateFlow<BindingInfo?>(null)

    val controller = _controllerRepo.getLiveController(_id)
    private val _controllerMappings = controller.map { it.mappings }

    val inputSummaries = Input.values().associate { it to getInputSummary(it) }

    fun startBinding(input: Input, multiple: Boolean) {
        _binding.value = BindingInfo(input, multiple)
    }

    fun persistKeyMapping(device: InputDevice, keyCode: Int): Boolean {
        val (input, multiple) = _binding.value ?: return false
        val mapping = KeyMapping(device.descriptor, input, keyCode)
        persistMapping(mapping, multiple)
        return true
    }

    fun persistAxisMapping(device: InputDevice, axis: Int, isNegative: Boolean): Boolean {
        val (input, multiple) = _binding.value ?: return false
        val mapping = AxisMapping(device.descriptor, input, axis, isNegative)
        persistMapping(mapping, multiple)
        return true
    }

    private fun persistMapping(mapping: Mapping, multiple: Boolean) {
        if (multiple) {
            _controllerRepo.addMapping(_id, mapping)
        } else {
            _controllerRepo.putMapping(_id, mapping)
        }
        _binding.value = null
    }

    private fun getInputSummary(input: Input): Flow<InputDisplay> {
        val mappings = _controllerMappings.map {
            it.filter { m -> m.input == input }
        }
        return _binding.combine(mappings) { currBinding, currMappings ->
            if (currBinding?.input == input) {
                if (currBinding.multiple) {
                    InputDisplay.Resource(R.string.input_menu_add_mapping)
                } else {
                    InputDisplay.Resource(R.string.input_menu_put_mapping)
                }
            } else if (currMappings.isEmpty()) {
                InputDisplay.Resource(R.string.input_menu_unmapped)
            } else {
                InputDisplay.Text(currMappings.joinToString(", "))
            }
        }
    }
}