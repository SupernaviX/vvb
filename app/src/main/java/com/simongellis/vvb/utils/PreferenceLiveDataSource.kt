package com.simongellis.vvb.utils

import android.content.SharedPreferences
import androidx.lifecycle.LifecycleOwner
import androidx.lifecycle.LiveData
import androidx.lifecycle.Observer

@Suppress("UNCHECKED_CAST")
class PreferenceLiveDataSource(private val preferences: SharedPreferences) :
    SharedPreferences.OnSharedPreferenceChangeListener {
    private var _activeListeners = 0
    private val _liveData = HashMap<String, PreferenceLiveData<*>>()

    fun <T> get(key: String, mapper: () -> T): PreferenceLiveData<T> {
        if (_liveData.contains(key)) {
            return _liveData[key] as PreferenceLiveData<T>
        }
        val data = PreferenceLiveData(mapper)
        _liveData[key] = data
        return data
    }

    inner class PreferenceLiveData<T>(private val mapper: () -> T): LiveData<T>(mapper()) {
        fun update() {
            value = mapper()
        }

        override fun getValue(): T {
            return super.getValue()!!
        }

        override fun onActive() {
            addActive()
        }

        override fun onInactive() {
            removeActive()
        }

        override fun observe(owner: LifecycleOwner, observer: Observer<in T>) {
            super.observe(owner, observer)
            // The caller should immediately see the initial value
            observer.onChanged(value)
        }
    }

    private fun addActive() {
        if (_activeListeners == 0) {
            preferences.registerOnSharedPreferenceChangeListener(this)
        }
        ++_activeListeners
    }
    private fun removeActive() {
        --_activeListeners
        if (_activeListeners == 0) {
            preferences.unregisterOnSharedPreferenceChangeListener(this)
        }
    }

    override fun onSharedPreferenceChanged(preferences: SharedPreferences, key: String) {
        _liveData[key]?.update()
    }
}