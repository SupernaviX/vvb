package com.simongellis.vvb.utils

import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleOwner
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.repeatOnLifecycle
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.collect
import kotlinx.coroutines.flow.takeWhile
import kotlinx.coroutines.launch

/**
 * Run a function with the value of a StateFlow, in harmony with this component's lifecycle
 * 1. Run it once with the initial value synchronously
 * 2. Start listening for changes when this component is Started
 * 3. Stop listening for changes when this component is Stopped
 */
fun <T> LifecycleOwner.observeEager(flow: Flow<T>, observer: (T) -> Unit) {
    lifecycleScope.launch {
        // Grab the first value out of this flow immediately
        var done = false
        flow.takeWhile { !done }.collect {
            observer(it)
            done = true
        }
        repeatOnLifecycle(Lifecycle.State.STARTED) {
            flow.collect { observer(it) }
        }
    }
}

/**
 * Run a function with the value of a SharedFlow, in harmony with this component's lifecycle
 * 1. Start listening for events when this component is Started
 * 2. Stop listening for events when this component is Stopped
 */
fun <T> LifecycleOwner.observe(flow: Flow<T>, observer: (T) -> Unit) {
    lifecycleScope.launch {
        repeatOnLifecycle(Lifecycle.State.STARTED) {
            flow.collect { observer(it) }
        }
    }
}