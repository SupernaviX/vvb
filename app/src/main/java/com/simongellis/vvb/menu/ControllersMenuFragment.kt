package com.simongellis.vvb.menu

import android.content.SharedPreferences
import android.os.Bundle
import androidx.preference.Preference
import androidx.preference.PreferenceCategory
import androidx.preference.PreferenceFragmentCompat
import com.simongellis.vvb.R
import com.simongellis.vvb.game.ControllerDao

class ControllersMenuFragment: PreferenceFragmentCompat(), SharedPreferences.OnSharedPreferenceChangeListener {
    private var _isDeleting = false
    private val _controllerDao by lazy {
        ControllerDao(preferenceManager.sharedPreferences)
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        preferenceManager.sharedPreferences.registerOnSharedPreferenceChangeListener(this)
    }

    override fun onCreatePreferences(savedInstanceState: Bundle?, rootKey: String?) {
        val prefScreen = preferenceManager.createPreferenceScreen(context)

        prefScreen.addPreference(PreferenceCategory(context).apply {
            key = "controllers"
            setTitle(R.string.controller_menu_controllers)
        })

        val manageCategory = PreferenceCategory(context).apply {
            key = "manage_controllers"
            setTitle(R.string.controller_menu_manage)
        }
        prefScreen.addPreference(manageCategory)
        manageCategory.addPreference(Preference(context).apply {
            key = "add_controller"
            setTitle(R.string.controller_menu_new)
            fragment = ControllerInputMenuFragment::class.qualifiedName
            setOnPreferenceClickListener {
                val controller = _controllerDao.addController()
                it.extras.putString("id", controller.id)
                false // returning false triggers default behavior of "load fragment"
            }
        })
        manageCategory.addPreference(Preference(context).apply {
            key = "delete_controller"
            setTitle(R.string.controller_menu_delete)
            setOnPreferenceClickListener {
                setDeleting(!_isDeleting)
                true
            }
        })

        preferenceScreen = prefScreen

        updateControllerPreferences()
    }

    override fun onSharedPreferenceChanged(sharedPreferences: SharedPreferences?, key: String?) {
        if (key === "controllers") {
            updateControllerPreferences()
        }
    }

    private fun updateControllerPreferences() {
        val controllerCategory = findPreference<PreferenceCategory>("controllers")!!
        controllerCategory.removeAll()
        for (controller in _controllerDao.getControllers().sortedBy { it.name }) {
            val controllerPref = Preference(context).apply {
                key = controller.id
                title = controller.name
                fragment = ControllerInputMenuFragment::class.qualifiedName
                extras.apply {
                    putString("id", controller.id)
                }
                setOnPreferenceClickListener {
                    if (_isDeleting) {
                        _controllerDao.deleteController(controller)
                        setDeleting(false)
                        true
                    } else {
                        false
                    }
                }
            }
            controllerCategory.addPreference(controllerPref)
        }
    }

    override fun onResume() {
        super.onResume()
        requireActivity().setTitle(R.string.main_menu_controller_setup)
    }

    private fun setDeleting(value: Boolean) {
        _isDeleting = value
        findPreference<Preference>("delete_controller")?.apply {
            setTitle(if (value) {
                R.string.controller_menu_choose_delete
            } else {
                R.string.controller_menu_delete
            })
        }
    }
}