package com.simongellis.vvb.menu

import androidx.lifecycle.MutableLiveData
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel

class ControllerNameViewModel(savedStateHandle: SavedStateHandle): ViewModel() {
    val name = MutableLiveData<String>(savedStateHandle.get("initialValue"))
}