package com.simongellis.vvb

import android.media.AudioManager
import android.os.Bundle
import android.view.KeyEvent
import android.view.MotionEvent
import android.view.View
import androidx.appcompat.app.AppCompatActivity
import androidx.core.content.ContextCompat
import androidx.fragment.app.Fragment
import androidx.preference.Preference
import androidx.preference.PreferenceFragmentCompat
import com.simongellis.vvb.menu.MainMenuFragment

class MainActivity : AppCompatActivity(), PreferenceFragmentCompat.OnPreferenceStartFragmentCallback {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.main_activity)

        // Only run initialization once
        if (savedInstanceState == null) {
            val audio = ContextCompat.getSystemService(baseContext, AudioManager::class.java)!!
            val sampleRate = audio.getProperty(AudioManager.PROPERTY_OUTPUT_SAMPLE_RATE).toInt()
            val framesPerBurst = audio.getProperty(AudioManager.PROPERTY_OUTPUT_FRAMES_PER_BUFFER).toInt()
            nativeInitialize(sampleRate, framesPerBurst)

            supportFragmentManager
                .beginTransaction()
                .replace(R.id.fragment_container, MainMenuFragment())
                .commit()
        }
    }

    override fun onPreferenceStartFragment(caller: PreferenceFragmentCompat, pref: Preference): Boolean {
        displayFragment(pref.fragment, pref.extras)
        return true
    }

    inline fun <reified T: Fragment> displayFragment(args: Bundle?) {
        displayFragment(T::class.qualifiedName!!, args ?: Bundle())
    }

    fun displayFragment(fragmentName: String, args: Bundle) {
        val fragment = supportFragmentManager.fragmentFactory.instantiate(
            classLoader,
            fragmentName
        )
        fragment.arguments = args
        supportFragmentManager.beginTransaction()
            .replace(R.id.fragment_container, fragment)
            .addToBackStack(null)
            .commit()
    }

    override fun dispatchKeyEvent(event: KeyEvent): Boolean {
        val fragment = supportFragmentManager.fragments.firstOrNull()
        if (fragment is View.OnKeyListener) {
            if (fragment.onKey(fragment.view, event.keyCode, event)) {
                return true
            }
        }
        return super.dispatchKeyEvent(event)
    }

    override fun dispatchGenericMotionEvent(event: MotionEvent): Boolean {
        val fragment = supportFragmentManager.fragments.firstOrNull()
        if (fragment is View.OnGenericMotionListener) {
            if (fragment.onGenericMotion(fragment.view, event)) {
                return true
            }
        }
        return super.dispatchGenericMotionEvent(event)
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