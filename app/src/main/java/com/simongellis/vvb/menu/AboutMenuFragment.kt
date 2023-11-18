package com.simongellis.vvb.menu

import android.widget.TextView
import androidx.fragment.app.Fragment
import com.simongellis.vvb.BuildConfig
import com.simongellis.vvb.R

class AboutMenuFragment: Fragment(R.layout.about_view) {
    override fun onResume() {
        super.onResume()
        requireActivity().setTitle(R.string.main_menu_about)
        val version = view?.findViewById<TextView>(R.id.app_version) ?: return
        version.text = BuildConfig.VERSION_NAME
    }
}