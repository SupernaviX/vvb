package com.simongellis.vvb.menu

import android.content.Context
import android.content.DialogInterface
import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.ArrayAdapter
import android.widget.TextView
import androidx.appcompat.app.AlertDialog
import androidx.core.os.bundleOf
import androidx.preference.PreferenceDialogFragmentCompat

class DetailedListPreferenceDialogFragment: PreferenceDialogFragmentCompat() {
    private var _selectedIndex = -1

    private val preference
        get() = getPreference() as DetailedListPreference

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        _selectedIndex = savedInstanceState?.getInt(SAVE_STATE_INDEX, 0)
            ?: preference.findIndexOfValue(preference.value)
    }

    override fun onSaveInstanceState(outState: Bundle) {
        super.onSaveInstanceState(outState)
        outState.putInt(SAVE_STATE_INDEX, _selectedIndex)
    }

    override fun onPrepareDialogBuilder(builder: AlertDialog.Builder) {
        super.onPrepareDialogBuilder(builder)

        val adapter = ListAdapter(requireContext(), preference.detailedEntries)

        builder.setSingleChoiceItems(adapter, _selectedIndex) { dialog, which ->
            _selectedIndex = which
            onClick(dialog, DialogInterface.BUTTON_POSITIVE)
            dialog.dismiss()
        }
        builder.setPositiveButton(null, null)
    }

    override fun onDialogClosed(positiveResult: Boolean) {
        val entries = preference.detailedEntries
        if (positiveResult && _selectedIndex >= 0 && _selectedIndex < entries.size) {
            val entry = entries[_selectedIndex]
            if (preference.callChangeListener(entry.value)) {
                preference.value = entry.value
            }
        }
    }

    class ListAdapter(context: Context, private val entries: List<DetailedListPreference.Entry>) : ArrayAdapter<DetailedListPreference.Entry>(context, 0, entries) {
        override fun getView(position: Int, convertView: View?, parent: ViewGroup): View {
            val view = convertView ?: LayoutInflater.from(context).inflate(android.R.layout.simple_list_item_2, parent, false)

            val entry = entries[position]
            val line1 = view.findViewById<TextView>(android.R.id.text1)
            line1.text = entry.summary
            val line2 = view.findViewById<TextView>(android.R.id.text2)
            line2.text = entry.description
            return view
        }
    }

    companion object {
        const val SAVE_STATE_INDEX = "DetailedListPreferenceDialogFragment.index"

        fun newInstance(key: String): DetailedListPreferenceDialogFragment {
            val fragment = DetailedListPreferenceDialogFragment()
            fragment.arguments = bundleOf(ARG_KEY to key)
            return fragment
        }
    }
}