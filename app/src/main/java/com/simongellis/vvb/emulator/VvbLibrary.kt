package com.simongellis.vvb.emulator

import android.app.Activity
import android.content.Context
import android.media.AudioManager
import androidx.core.content.ContextCompat

class VvbLibrary {
    private var initialized = false

    fun initialize(activity: Activity) {
        if (initialized) { return }

        val audio = ContextCompat.getSystemService(activity, AudioManager::class.java)!!
        val sampleRate = audio.getProperty(AudioManager.PROPERTY_OUTPUT_SAMPLE_RATE).toInt()
        val framesPerBurst = audio.getProperty(AudioManager.PROPERTY_OUTPUT_FRAMES_PER_BUFFER).toInt()
        nativeInitialize(activity, sampleRate, framesPerBurst)

        initialized = true
    }

    fun changeDeviceParams() {
        nativeChangeDeviceParams()
    }

    private external fun nativeInitialize(context: Context, sampleRate: Int, framesPerBurst: Int)
    private external fun nativeChangeDeviceParams()

    companion object {
        val instance by lazy { VvbLibrary() }
    }
}