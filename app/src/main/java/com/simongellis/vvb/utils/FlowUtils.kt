package com.simongellis.vvb.utils

import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleOwner
import androidx.lifecycle.flowWithLifecycle
import com.fredporciuncula.flow.preferences.Preference
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.*

/**
 * Turn Preference data into a StateFlow, which always has a defined value
 */
fun <TRaw, TParsed> Preference<TRaw>.asStateFlow(
    scope: CoroutineScope,
    deserializer: (TRaw) -> TParsed
): StateFlow<TParsed> {
    return asFlow()
        .map { deserializer(it) }
        .stateIn(scope, SharingStarted.WhileSubscribed(), deserializer(get()))
}

/**
 * Run a function with the value of a flow, in harmony with this component's lifecycle
 * 1. Run it once with the initial value synchronously
 * 2. Start listening for changes when this component is Started
 * 3. Stop listening for changes when this component is Stopped
 */
suspend fun <T> LifecycleOwner.observe(flow: StateFlow<T>, observer: (T) -> Unit) {
    observer(flow.value)
    flow
        .flowWithLifecycle(lifecycle, Lifecycle.State.STARTED)
        .collect { observer(it) }
}