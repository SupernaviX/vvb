package com.simongellis.vvb.leia

import android.content.Context
import com.leia.sdk.LeiaSDK

@Suppress("unused")
class CNSDKLeiaAdapter(context: Context) : LeiaAdapter {
    override val leiaVersion = LeiaVersion.CNSDK

    private val sdk by lazy {
        val initArgs = LeiaSDK.InitArgs()
        initArgs.platform.context = context
        initArgs.enableFaceTracking = true
        initArgs.requiresFaceTrackingPermissionCheck = false
        LeiaSDK.createSDK(initArgs)
    }

    override fun enableBacklight() {
        sdk.enableBacklight(true)
    }

    override fun disableBacklight() {
        sdk.enableBacklight(false)
    }

    override fun registerBacklightListener(listener: LeiaAdapter.BacklightListener) {
        // This SDK doesn't let you listen for backlight mode changes.
        // It also doesn't require you to; it handles falling back to 2D itself.
    }
}