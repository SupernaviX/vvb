package com.simongellis.vvb

import android.os.Bundle
import android.view.KeyEvent
import android.view.MotionEvent
import android.view.View
import androidx.activity.viewModels
import androidx.appcompat.app.AppCompatActivity
import androidx.fragment.app.Fragment
import androidx.preference.Preference
import androidx.preference.PreferenceFragmentCompat
import com.simongellis.vvb.emulator.VvbLibrary
import com.simongellis.vvb.menu.DetailedListPreference
import com.simongellis.vvb.menu.DetailedListPreferenceDialogFragment
import com.simongellis.vvb.menu.MainMenuFragment
import com.simongellis.vvb.menu.GameMenuFragment

class MainActivity : AppCompatActivity(R.layout.main_activity),
    PreferenceFragmentCompat.OnPreferenceStartFragmentCallback,
    PreferenceFragmentCompat.OnPreferenceDisplayDialogCallback {
    private val viewModel: MainViewModel by viewModels()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // Only run initialization once
        if (savedInstanceState == null) {
            VvbLibrary.instance.initialize(this)

            supportFragmentManager
                .beginTransaction()
                .replace(R.id.fragment_container, MainMenuFragment())
                .setReorderingAllowed(true)
                .commit()
        }
    }

    override fun onResume() {
        super.onResume()
        if (viewModel.lastEvent.compareAndSet(MainViewModel.GameEvent.Opened, null)) {
            // If we just returned from a game, show the Game Actions menu
            closeAllSubMenus()
            displayFragment<GameMenuFragment>(null)
        }
    }

    override fun onPreferenceStartFragment(
        caller: PreferenceFragmentCompat,
        pref: Preference
    ): Boolean {
        pref.fragment?.let { displayFragment(it, pref.extras) }
        return true
    }

    override fun onPreferenceDisplayDialog(
        caller: PreferenceFragmentCompat,
        preference: Preference
    ): Boolean {
        return if (preference is DetailedListPreference) {
            val dialogFragment = DetailedListPreferenceDialogFragment.newInstance(preference.key)
            @Suppress("DEPRECATION")
            dialogFragment.setTargetFragment(caller, 0)
            dialogFragment.show(
                caller.parentFragmentManager,
                "androidx.preference.PreferenceFragment.DIALOG"
            )
            true
        } else {
            false
        }
    }

    inline fun <reified T : Fragment> displayFragment(args: Bundle?) {
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
            .setReorderingAllowed(true)
            .commit()
    }

    fun closeAllSubMenus() {
        while (supportFragmentManager.backStackEntryCount > 0) {
            supportFragmentManager.popBackStackImmediate()
        }
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
}