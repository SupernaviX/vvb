package com.simongellis.vvb.utils

import androidx.lifecycle.*

class LiveEvent<T> {
    private data class Event<T>(val value: T) {
        var seen = false
    }
    private val _data = MutableLiveData<Event<T>>()
    fun observe(owner: LifecycleOwner, observer: Observer<T>) {
        _data.observe(owner, Observer {
            if (!it.seen) {
                it.seen = true
                observer.onChanged(it.value)
            }
        })
    }
    fun emit(value: T) {
        _data.value = Event(value)
    }
}