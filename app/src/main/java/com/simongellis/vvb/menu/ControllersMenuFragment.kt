package com.simongellis.vvb.menu

import android.os.Bundle
import androidx.core.os.bundleOf
import androidx.fragment.app.viewModels
import androidx.preference.Preference
import androidx.preference.PreferenceCategory
import androidx.preference.PreferenceFragmentCompat
import com.simongellis.vvb.MainActivity
import com.simongellis.vvb.R
import com.simongellis.vvb.game.ControllerDao
import com.simongellis.vvb.utils.observe
import com.simongellis.vvb.utils.observeEager

class ControllersMenuFragment: PreferenceFragmentCompat() {
    private val viewModel: ControllersViewModel by viewModels()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        observeEager(viewModel.controllers) { updateControllerList(it) }
        observe(viewModel.editingController) {
            editControllerMappings(it.id)
        }
        observeEager(viewModel.renameLabel) {
            findPreference<Preference>("rename_controller")?.setTitle(it)
        }
        observeEager(viewModel.deleteLabel) {
            findPreference<Preference>("delete_controller")?.setTitle(it)
        }
        observe(viewModel.showNameDialog) { showControllerNameDialog(it) }
        observe(viewModel.showAutoMapDialog) { showAutoMapDialog() }
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
                viewModel.promptAutoMap()
                true
            }
        })
        manageCategory.addPreference(Preference(context).apply {
            key = "new_controller"
            setTitle(R.string.controller_menu_new_controller)
            setOnPreferenceClickListener {
                viewModel.promptAddController()
                true
            }
        })
        manageCategory.addPreference(Preference(context).apply {
            key = "rename_controller"
            setTitle(R.string.controller_menu_rename_controller)
            setOnPreferenceClickListener {
                viewModel.toggleRenaming()
                true
            }
        })
        manageCategory.addPreference(Preference(context).apply {
            key = "delete_controller"
            setTitle(R.string.controller_menu_delete_controller)
            setOnPreferenceClickListener {
                viewModel.toggleDeleting()
                true
            }
        })

        preferenceScreen = prefScreen
    }

    private fun updateControllerList(controllers: List<ControllerDao.Controller>) {
        // This method is triggered on preference change, so it can run after the fragment dies.
        // Bail early if this has happened to avoid calamity
        val context = context ?: return

        val controllerCategory = findPreference<PreferenceCategory>("controllers")!!
        controllerCategory.removeAll()
        for (controller in controllers.sortedBy { it.name }) {
            val controllerPref = Preference(context).apply {
                key = controller.id
                title = controller.name
                setOnPreferenceClickListener {
                    viewModel.doAction(controller)
                    true
                }
            }
            controllerCategory.addPreference(controllerPref)
        }
    }

    private fun editControllerMappings(id: String) {
        val starter = activity as MainActivity
        starter.displayFragment<ControllerInputMenuFragment>(bundleOf("id" to id))
    }

    private fun showAutoMapDialog() {
        val newDialog = AutoMapDialog()
        newDialog.show(childFragmentManager, "auto_mapper")
    }

    private fun showControllerNameDialog(nameDialog: ControllersViewModel.NameDialog) {
        val newDialog = ControllerNameDialog.newInstance(
            nameDialog.action,
            nameDialog.initialValue
        )
        newDialog.show(childFragmentManager, "controller_name")
    }
}