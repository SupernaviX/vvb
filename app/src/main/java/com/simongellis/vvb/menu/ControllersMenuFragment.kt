package com.simongellis.vvb.menu

import android.content.SharedPreferences
import android.os.Bundle
import androidx.core.content.edit
import androidx.preference.Preference
import androidx.preference.PreferenceCategory
import androidx.preference.PreferenceFragmentCompat
import com.simongellis.vvb.R
import java.util.*
import kotlin.collections.HashSet

class ControllersMenuFragment: PreferenceFragmentCompat(), SharedPreferences.OnSharedPreferenceChangeListener {
    data class Controller(val id: String, val name: String) {
        val descriptor get() = "$id::$name"

        companion object {
            fun fromDescriptor(descriptor: String): Controller {
                val (id, name) = descriptor.split("::", limit = 2)
                return Controller(id, name)
            }
        }
    }

    private var _isDeleting = false

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
                val controller = addController()
                it.extras.putString("id", controller.id)
                it.extras.putString("name", controller.name)
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
        for (controller in getControllers().sortedBy { it.name }) {
            val controllerPref = Preference(context).apply {
                key = controller.id
                title = controller.name
                fragment = ControllerInputMenuFragment::class.qualifiedName
                extras.apply {
                    putString("id", controller.id)
                    putString("name", controller.name)
                }
                setOnPreferenceClickListener {
                    if (_isDeleting) {
                        deleteController(controller)
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

    private fun getControllers(): List<Controller> {
        return getDescriptors().map { Controller.fromDescriptor(it) }
    }

    private fun addController(): Controller {
        val descriptors = getDescriptors()

        val id = UUID.randomUUID().toString()
        val name = "Controller ${descriptors.size + 1}"
        val controller = Controller(id, name)

        preferenceManager.sharedPreferences.edit {
            putStringSet("controllers", HashSet(descriptors).apply { add(controller.descriptor) })
        }

        return Controller(id, name)
    }

    private fun deleteController(controller: Controller) {
        val descriptors = getDescriptors()

        preferenceManager.sharedPreferences.edit {
            putStringSet("controllers", HashSet(descriptors).apply { remove(controller.descriptor) })
        }
    }

    private fun getDescriptors(): Set<String> {
        return preferenceManager.sharedPreferences.getStringSet("controllers", setOf())!!
    }

}