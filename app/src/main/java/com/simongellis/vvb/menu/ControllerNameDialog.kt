package com.simongellis.vvb.menu

import android.app.AlertDialog
import android.app.Dialog
import android.os.Bundle
import android.view.LayoutInflater
import androidx.annotation.StringRes
import androidx.core.os.bundleOf
import androidx.core.widget.addTextChangedListener
import androidx.fragment.app.DialogFragment
import androidx.fragment.app.viewModels
import com.simongellis.vvb.R
import com.simongellis.vvb.databinding.TextBoxBinding

class ControllerNameDialog: DialogFragment() {
    private val viewModel: ControllerNameViewModel by viewModels()
    private val parentViewModel: ControllersViewModel by viewModels({ requireParentFragment() })
    private val action by lazy {
        requireArguments().getInt("action")
    }

    override fun onCreateDialog(savedInstanceState: Bundle?): Dialog {
        val context = requireContext()
        val layoutInflater = LayoutInflater.from(context)
        val view = TextBoxBinding.inflate(layoutInflater)
        val input = view.input.apply {
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

        return AlertDialog.Builder(context)
            .setTitle(R.string.controller_menu_name)
            .setView(view.root)
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