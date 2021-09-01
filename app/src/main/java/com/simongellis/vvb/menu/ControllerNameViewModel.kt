package com.simongellis.vvb.menu

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import kotlinx.coroutines.flow.MutableStateFlow

class ControllerNameViewModel(savedStateHandle: SavedStateHandle): ViewModel() {
    val name = MutableStateFlow(savedStateHandle.get("initialValue") ?: "")
}