package com.simongellis.vvb.menu

import android.app.AlertDialog
import android.app.Dialog
import android.os.Bundle
import android.text.InputType
import android.widget.EditText
import androidx.annotation.StringRes
import androidx.core.os.bundleOf
import androidx.core.widget.addTextChangedListener
import androidx.fragment.app.DialogFragment
import androidx.fragment.app.viewModels
import com.simongellis.vvb.R

class ControllerNameDialog: DialogFragment() {
    private val viewModel: ControllerNameViewModel by viewModels()
    private val parentViewModel: ControllersViewModel by viewModels({ requireParentFragment() })
    private val action by lazy {
        requireArguments().getInt("action")
    }

    override fun onCreateDialog(savedInstanceState: Bundle?): Dialog {
        val input = EditText(requireContext()).apply {
            inputType = InputType.TYPE_CLASS_TEXT
            addTextChangedListener {
                viewModel.name.value = text.toString()
            }
        }
        viewModel.name.observe(this) {
            if (input.text.toString() != it) {
                input.setText(it)
                input.selectAll()
            }
        }

        return AlertDialog.Builder(requireContext())
            .setTitle(R.string.controller_menu_name)
            .setView(input)
            .setPositiveButton(action) { _, _ ->
                parentViewModel.chooseControllerName(input.text.toString())
                dismiss()
            }
            .setNegativeButton(R.string.controller_menu_cancel) { _, _ ->
                dismiss()
            }
            .create()
    }

    companion object {
        fun newInstance(@StringRes action: Int, initialValue: String): ControllerNameDialog {
            return ControllerNameDialog().apply {
                arguments = bundleOf(
                    "action" to action,
                    "initialValue" to initialValue
                )
            }
        }
    }
}