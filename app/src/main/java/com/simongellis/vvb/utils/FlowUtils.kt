package com.simongellis.vvb.utils

import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleOwner
import androidx.lifecycle.flowWithLifecycle
import androidx.lifecycle.lifecycleScope
import com.fredporciuncula.flow.preferences.Preference
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.launch

/**
 * Turn Preference data into a StateFlow, which always has a defined value
 */
fun <TRaw, TParsed> Preference<TRaw>.asStateFlow(
    scope: CoroutineScope,
    deserializer: (TRaw) -> TParsed
): StateFlow<TParsed> {
    return asFlow()
        .map { deserializer(it) }
        .asStateFlow(scope, deserializer(get()))
}

fun <TIn, TOut> StateFlow<TIn>.mapAsState(
    scope: CoroutineScope,
    transform: (TIn) -> TOut
): StateFlow<TOut> {
    return map { transform(it) }.asStateFlow(scope, transform(value))
}

fun <T> Flow<T>.asStateFlow(scope: CoroutineScope, value: T): StateFlow<T> {
    return stateIn(scope, SharingStarted.WhileSubscribed(), value)
}

/**
 * Run a function with the value of a flow, in harmony with this component's lifecycle
 * 1. Run it once with the initial value synchronously
 * 2. Start listening for changes when this component is Started
 * 3. Stop listening for changes when this component is Stopped
 */
fun <T> LifecycleOwner.observe(flow: StateFlow<T>, observer: (T) -> Unit) {
    lifecycleScope.launch {
        observer(flow.value)
        flow
            .flowWithLifecycle(lifecycle, Lifecycle.State.STARTED)
            .collect { observer(it) }
    }
}