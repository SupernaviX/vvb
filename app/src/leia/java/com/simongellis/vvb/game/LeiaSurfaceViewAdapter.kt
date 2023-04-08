package com.simongellis.vvb.game

import android.content.Context
import android.util.AttributeSet
import android.util.Log
import com.leia.sdk.LeiaSDK
import com.leia.sdk.views.InterlacedSurfaceView

class LeiaSurfaceViewAdapter : InterlacedSurfaceView, SurfaceViewAdapter, LeiaSDK.Delegate {
    constructor(context: Context) : super(context)
    constructor(context: Context, attrs: AttributeSet?) : super(context, attrs)

    private var leiaRendererAdapter: LeiaRendererAdapter? = null

    init {
        val initArgs = LeiaSDK.InitArgs()
        initArgs.delegate = this
        initArgs.platform.context = context.applicationContext
        initArgs.platform.activity = getActivity(context)
        initArgs.enableFaceTracking = true
        LeiaSDK.createSDK(initArgs)
    }

    override fun setRenderer(renderer: Renderer) {
        // wrap the InterlacedRenderer instance which the parent class constructed
        val adapter = LeiaRendererAdapter(renderer)
        leiaRendererAdapter = adapter
        super.setRenderer(adapter)
        setViewAsset(adapter.asset)
    }

    override fun setRenderer(renderer: com.simongellis.vvb.emulator.Renderer) {
        leiaRendererAdapter?.innerRenderer = renderer
    }

    override fun didInitialize(sdk: LeiaSDK) {
        Log.i("LeiaSurfaceViewAdapter", "didInitialize")
        sdk.enableBacklight(true)
    }

    override fun onFaceTrackingFatalError(sdk: LeiaSDK) {
        Log.i("LeiaSurfaceViewAdapter", "onFaceTrackingFatalError")
        sdk.isFaceTrackingInFatalError?.let {
            Log.i("LeiaSurfaceViewAdapter", "${it.code}: ${it.message}")
        }
    }

    override fun onFaceTrackingStarted(sdk: LeiaSDK) {
        Log.i("LeiaSurfaceViewAdapter", "onFaceTrackingStarted")
    }

    override fun onFaceTrackingStopped(sdk: LeiaSDK) {
        Log.i("LeiaSurfaceViewAdapter", "onFaceTrackingStopped")
    }
}