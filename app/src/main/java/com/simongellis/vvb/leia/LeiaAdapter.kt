package com.simongellis.vvb.leia

import android.content.Context
import android.content.pm.PackageManager

interface LeiaAdapter {
    val leiaVersion: LeiaVersion?
    fun enableBacklight()
    fun disableBacklight()
    fun registerBacklightListener(listener: BacklightListener)

    interface BacklightListener {
        fun onBacklightChanged(enabled: Boolean)
    }

    companion object {
        private var instance: LeiaAdapter? = null
        fun instance(context: Context): LeiaAdapter {
            return instance ?: newInstance(context).also { instance = it }
        }

        private fun newInstance(context: Context): LeiaAdapter {
            val clazz = try {
                getImplementation(context)
            } catch (ex: Exception) {
                LegacyLeiaAdapter::class.java
            }
            val ctor = clazz.getConstructor(Context::class.java)
            return ctor.newInstance(context.applicationContext)
        }

        private fun getImplementation(context: Context): Class<out LeiaAdapter> {
            val metadata = context.packageManager.getApplicationInfo(
                context.packageName, PackageManager.GET_META_DATA
            ).metaData
            val name = metadata.getString("com.simongellis.vvb.leia.LeiaAdapter")
                ?: return LegacyLeiaAdapter::class.java
            @Suppress("UNCHECKED_CAST") return Class.forName(name) as Class<out LeiaAdapter>
        }
    }
}