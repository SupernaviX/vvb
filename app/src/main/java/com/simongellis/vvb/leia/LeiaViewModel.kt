package com.simongellis.vvb.leia

import android.app.Activity
import android.content.pm.PackageManager
import androidx.activity.ComponentActivity
import androidx.fragment.app.Fragment
import androidx.lifecycle.ViewModel
import androidx.lifecycle.ViewModelProvider

class LeiaViewModel : ViewModel() {
    private var adapter: LeiaAdapter? = null

    fun getAdapter(activity: Activity): LeiaAdapter {
        return adapter ?: initAdapter(activity)
    }

    private fun initAdapter(activity: Activity): LeiaAdapter {
        return buildAdapter(activity).also { adapter = it }
    }

    private fun buildAdapter(activity: Activity): LeiaAdapter {
        val clazz = getImplementation(activity)
        val ctor = clazz.getConstructor(Activity::class.java)
        return ctor.newInstance(activity)
    }

    companion object {
        private var impl: Class<out LeiaAdapter>? = null
        fun getImplementation(activity: Activity): Class<out LeiaAdapter> {
            return impl ?: try {
                @Suppress("DEPRECATION") val metadata = activity.packageManager.getApplicationInfo(
                    activity.packageName, PackageManager.GET_META_DATA
                ).metaData
                val name = metadata.getString("com.simongellis.vvb.leia.LeiaAdapter")
                    ?: return LegacyLeiaAdapter::class.java
                @Suppress("UNCHECKED_CAST") return Class.forName(name) as Class<out LeiaAdapter>
            } catch (ex: Exception) {
                LegacyLeiaAdapter::class.java
            }.also { impl = it }
        }

        val Activity.leiaAdapter: LeiaAdapter
            get() {
                if (this !is ComponentActivity) {
                    throw Exception("Unexpected non-component activity")
                }
                val provider = ViewModelProvider(this)
                val model = provider[LeiaViewModel::class.java]
                return model.getAdapter(this)
            }

        val Fragment.leiaAdapter: LeiaAdapter
            get() = activity!!.leiaAdapter
    }

}