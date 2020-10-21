package com.simongellis.vvb

import android.media.AudioManager
import android.os.Bundle
import android.view.KeyEvent
import androidx.appcompat.app.AppCompatActivity
import androidx.core.content.ContextCompat
import androidx.preference.Preference
import androidx.preference.PreferenceFragmentCompat

class MainActivity : AppCompatActivity(), PreferenceFragmentCompat.OnPreferenceStartFragmentCallback {
    private var _pointer = 0L

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val audio = ContextCompat.getSystemService(applicationContext, AudioManager::class.java)!!
        val sampleRate = audio.getProperty(AudioManager.PROPERTY_OUTPUT_SAMPLE_RATE).toInt()
        val framesPerBurst = audio.getProperty(AudioManager.PROPERTY_OUTPUT_FRAMES_PER_BUFFER).toInt()

        nativeInitialize(sampleRate, framesPerBurst)
        setContentView(R.layout.activity_main)
        supportFragmentManager
            .beginTransaction()
            .replace(R.id.fragment_container, MainMenuFragment())
            .commit()
    }

    override fun onPreferenceStartFragment(caller: PreferenceFragmentCompat, pref: Preference): Boolean {
        val args = pref.extras
        val fragment = supportFragmentManager.fragmentFactory.instantiate(
            classLoader,
            pref.fragment
        )
        fragment.arguments = args
        fragment.setTargetFragment(caller, 0)
        supportFragmentManager.beginTransaction()
            .replace(R.id.fragment_container, fragment)
            .addToBackStack(null)
            .commit()
        return true
    }

    override fun dispatchKeyEvent(event: KeyEvent): Boolean {
        val fragment = supportFragmentManager.fragments.firstOrNull()
        if (fragment is InputMenuFragment) {
            if (fragment.onKeyEvent(event)) {
                return true
            }
        }
        return super.dispatchKeyEvent(event)
    }

    fun changeDeviceParams() {
        nativeChangeDeviceParams()
    }

    companion object {
        init {
            System.loadLibrary("vvb")
        }
    }

    private external fun nativeInitialize(sampleRate: Int, framesPerBurst: Int)
    private external fun nativeChangeDeviceParams()
}