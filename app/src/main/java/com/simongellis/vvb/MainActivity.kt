package com.simongellis.vvb

import android.os.Bundle
import androidx.appcompat.app.AppCompatActivity

class MainActivity : AppCompatActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        nativeInitialize()
        setContentView(R.layout.activity_main)
    }

    fun changeDeviceParams() {
        nativeChangeDeviceParams()
    }

    companion object {
        init {
            System.loadLibrary("vvb")
        }
    }

    private external fun nativeInitialize()
    private external fun nativeChangeDeviceParams()
}