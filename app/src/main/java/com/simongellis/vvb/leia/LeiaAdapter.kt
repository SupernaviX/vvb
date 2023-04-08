package com.simongellis.vvb.leia

interface LeiaAdapter {
    val leiaVersion: LeiaVersion?
    fun enableBacklight()
    fun disableBacklight()
    fun registerBacklightListener(listener: BacklightListener)

    interface BacklightListener {
        fun onBacklightChanged(enabled: Boolean)
    }
}