package com.simongellis.vvb.leia

import android.app.Activity
import com.leia.sdk.LeiaSDK

@Suppress("unused")
class CNSDKLeiaAdapter(activity: Activity) : LeiaAdapter {
    override val leiaVersion = LeiaVersion.CNSDK

    private val sdk by lazy {
        val initArgs = LeiaSDK.InitArgs()
        initArgs.platform.context = activity.applicationContext
        initArgs.platform.activity = activity
        initArgs.enableFaceTracking = true
        LeiaSDK.createSDK(initArgs)
    }

    override fun enableBacklight() {
        sdk.enableBacklight(true)
    }

    override fun disableBacklight() {
        sdk.enableBacklight(false)
    }

    override fun registerBacklightListener(listener: LeiaAdapter.BacklightListener) {
        // This SDK doesn't let you listen for this. It also doesn't require you to.
    }
}