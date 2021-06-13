package com.simongellis.vvb.utils

import android.content.SharedPreferences
import androidx.lifecycle.LifecycleOwner
import androidx.lifecycle.LiveData
import androidx.lifecycle.Observer

class PreferenceLiveData<T>(
    private val preferences: SharedPreferences,
    private val prefKey: String,
    private val mapper: () -> T
): LiveData<T>(mapper()), SharedPreferences.OnSharedPreferenceChangeListener {
    override fun getValue(): T {
        return super.getValue()!!
    }

    override fun onActive() {
        preferences.registerOnSharedPreferenceChangeListener(this)
    }

    override fun onInactive() {
        preferences.unregisterOnSharedPreferenceChangeListener(this)
    }

    override fun observe(owner: LifecycleOwner, observer: Observer<in T>) {
        super.observe(owner, observer)
        // The caller should immediately see the initial value
        observer.onChanged(value)
    }

    override fun onSharedPreferenceChanged(sharedPreferences: SharedPreferences?, key: String?) {
        if (key == prefKey) {
            postValue(mapper())
        }
    }
}
