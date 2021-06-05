package com.simongellis.vvb.menu

import android.app.Dialog
import android.content.SharedPreferences
import android.os.Bundle
import android.text.InputType
import android.widget.EditText
import androidx.annotation.StringRes
import androidx.appcompat.app.AlertDialog
import androidx.preference.Preference
import androidx.preference.PreferenceCategory
import androidx.preference.PreferenceFragmentCompat
import com.simongellis.vvb.R
import com.simongellis.vvb.game.ControllerDao

class ControllersMenuFragment: PreferenceFragmentCompat(), SharedPreferences.OnSharedPreferenceChangeListener {
    private enum class State { Normal, Renaming, Deleting }
    private var _state = State.Normal
        set(value) {
            field = value
            findPreference<Preference>("rename_controller")?.apply {
                setTitle(
                    if (value == State.Renaming) {
                        R.string.controller_menu_choose_rename
                    } else {
                        R.string.controller_menu_rename_controller
                    }
                )
            }
            findPreference<Preference>("delete_controller")?.apply {
                setTitle(
                    if (value == State.Deleting) {
                        R.string.controller_menu_choose_delete
                    } else {
                        R.string.controller_menu_delete_controller
                    }
                )
            }
        }

    private var _dialog: Dialog? = null
        set(value) {
            field = value
            value?.setOnDismissListener { field = null }
        }
    private val _controllerDao by lazy {
        ControllerDao(preferenceManager.sharedPreferences)
    }
    private val _autoMapper = ControllerAutoMapper()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        preferenceManager.sharedPreferences.registerOnSharedPreferenceChangeListener(this)
    }

    override fun onPause() {
        super.onPause()
        _dialog?.apply { dismiss() }
    }

    override fun onResume() {
        super.onResume()
        requireActivity().setTitle(R.string.main_menu_controller_setup)
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
            key = "auto_map_new_controller"
            setTitle(R.string.controller_menu_automap_new_controller)
            setOnPreferenceClickListener {
                _state = State.Normal
                autoMapController()
                true
            }
        })
        manageCategory.addPreference(Preference(context).apply {
            key = "new_controller"
            setTitle(R.string.controller_menu_new_controller)
            setOnPreferenceClickListener {
                _state = State.Normal
                addController()
                true
            }
        })
        manageCategory.addPreference(Preference(context).apply {
            key = "rename_controller"
            setTitle(R.string.controller_menu_rename_controller)
            setOnPreferenceClickListener {
                toggleState(State.Renaming)
                true
            }
        })
        manageCategory.addPreference(Preference(context).apply {
            key = "delete_controller"
            setTitle(R.string.controller_menu_delete_controller)
            setOnPreferenceClickListener {
                toggleState(State.Deleting)
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
            val controllerPref = Preference(requireContext()).apply {
                key = controller.id
                title = controller.name
                fragment = ControllerInputMenuFragment::class.qualifiedName
                extras.apply {
                    putString("id", controller.id)
                }
                setOnPreferenceClickListener {
                    when (_state) {
                        State.Normal -> false // Causes it to open ControllerInputMenuFragment
                        State.Renaming -> {
                            renameController(controller)
                            _state = State.Normal
                            true
                        }
                        State.Deleting -> {
                            deleteController(controller)
                            _state = State.Normal
                            true
                        }
                    }
                }
            }
            controllerCategory.addPreference(controllerPref)
        }
    }

    private fun autoMapController() {
        _dialog = DeviceListDialog(requireContext()).apply {
            setDeviceFilter(_autoMapper::isMappable)
            setOnDeviceChosen { device ->
                val result = _autoMapper.computeMappings(device)
                val controller = _controllerDao.addController(result.name)
                for (mapping in result.mappings) {
                    _controllerDao.addMapping(controller.id, mapping)
                }
                updateControllerPreferences()
                if (!result.fullyMapped) {
                    findPreference<Preference>(controller.id)?.also { onPreferenceTreeClick(it) }
                }
            }
            show()
        }
    }

    private fun addController() {
        val controllerCount = _controllerDao.getControllers().size
        showControllerNameDialog(
            R.string.controller_menu_create,
            "Controller ${controllerCount + 1}"
        ) { name ->
            val controller = _controllerDao.addController(name)
            updateControllerPreferences()
            findPreference<Preference>(controller.id)?.also { onPreferenceTreeClick(it) }
        }
    }

    private fun renameController(controller: ControllerDao.Controller) {
        showControllerNameDialog(
            R.string.controller_menu_rename,
            controller.name
        ) { name ->
            val newController = ControllerDao.Controller(controller.id, name)
            _controllerDao.putController(newController)
        }
    }

    private fun deleteController(controller: ControllerDao.Controller) {
        _controllerDao.deleteController(controller)
    }

    private fun showControllerNameDialog(@StringRes action: Int, initialValue: String, callback: (name: String) -> Unit) {
        val input = EditText(requireContext()).apply {
            inputType = InputType.TYPE_CLASS_TEXT
            text.append(initialValue)
            selectAll()
        }
        _dialog = AlertDialog.Builder(requireContext())
            .setTitle(R.string.controller_menu_name)
            .setView(input)
            .setPositiveButton(action) { _, _ ->
                callback(input.text.toString())
            }
            .setNegativeButton(R.string.controller_menu_cancel) { dialog, _ ->
                dialog.cancel()
            }
            .show()
    }

    private fun toggleState(state: State) {
        _state = if (_state == state) {
            State.Normal
        } else {
            state
        }
    }
}