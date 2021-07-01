package com.simongellis.vvb.menu

import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView
import androidx.appcompat.app.AlertDialog
import androidx.core.os.bundleOf
import androidx.preference.PreferenceDialogFragmentCompat
import androidx.recyclerview.widget.LinearLayoutManager
import androidx.recyclerview.widget.RecyclerView
import com.simongellis.vvb.game.VideoMode


class VideoModePreferenceDialogFragment: PreferenceDialogFragmentCompat() {
    private val videoModePreference
        get() = preference as VideoModeDialogPreference
    private var selected: VideoMode? = null

    override fun onPrepareDialogBuilder(builder: AlertDialog.Builder) {
        val context = requireContext()
        val videoModeAdapter = VideoModeListAdapter(this::onModeChosen)
        val view = RecyclerView(context).apply {
            layoutManager = LinearLayoutManager(context)
            adapter = videoModeAdapter
        }
        builder.setView(view).setPositiveButton(null, null)
    }

    private fun onModeChosen(mode: VideoMode) {
        selected = mode
        dismiss()
    }

    override fun onDialogClosed(positiveResult: Boolean) {
        selected?.also {
            videoModePreference.setValue(it)
        }
    }

    class VideoModeListAdapter(private val onModeChosen: (VideoMode) -> Unit): RecyclerView.Adapter<VideoModeListAdapter.ViewHolder>() {
        private val videoModes = VideoMode.values()

        class ViewHolder(view: View): RecyclerView.ViewHolder(view) {
            var mode: VideoMode? = null
            val text1 = view.findViewById<TextView>(android.R.id.text1)!!
            val text2 = view.findViewById<TextView>(android.R.id.text2)!!
        }

        override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): ViewHolder {
            val view = LayoutInflater.from(parent.context).inflate(android.R.layout.simple_list_item_2, parent, false)
            val holder = ViewHolder(view)
            view.setOnClickListener {
                holder.mode?.apply(onModeChosen)
            }
            return holder
        }

        override fun onBindViewHolder(holder: ViewHolder, position: Int) {
            val mode = videoModes[position]
            holder.mode = mode
            holder.text1.setText(mode.summary)
            holder.text2.setText(mode.description)
        }

        override fun getItemCount(): Int {
            return videoModes.size
        }
    }

    companion object {
        fun newInstance(key: String): VideoModePreferenceDialogFragment {
            val fragment = VideoModePreferenceDialogFragment()
            fragment.arguments = bundleOf(ARG_KEY to key)
            return fragment
        }
    }
}
