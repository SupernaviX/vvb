package com.simongellis.vvb.leia

import android.content.Context
import com.leia.android.lights.LeiaDisplayManager
import com.leia.android.lights.LeiaSDK

class LegacyLeiaAdapter(context: Context) : LeiaAdapter {
    private val manager = LeiaSDK.getDisplayManager(context)
    private val listeners = mutableSetOf<LeiaAdapter.BacklightListener>()

    init {
        manager?.registerBacklightModeListener { mode ->
            listeners.forEach {
                it.onBacklightChanged(mode == LeiaDisplayManager.BacklightMode.MODE_3D)
            }
        }
    }

    override val leiaVersion: LeiaVersion? = if (manager != null) {
        LeiaVersion.Legacy
    } else {
        null
    }

    override fun enableBacklight() {
        manager?.requestBacklightMode(LeiaDisplayManager.BacklightMode.MODE_3D)
    }

    override fun disableBacklight() {
        manager?.requestBacklightMode(LeiaDisplayManager.BacklightMode.MODE_2D)
    }

    override fun registerBacklightListener(listener: LeiaAdapter.BacklightListener) {
        listeners.add(listener)
    }
}