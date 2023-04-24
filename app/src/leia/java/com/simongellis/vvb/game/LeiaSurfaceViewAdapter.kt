package com.simongellis.vvb.game

import android.content.Context
import android.util.AttributeSet
import com.leia.sdk.views.InterlacedSurfaceView

class LeiaSurfaceViewAdapter : InterlacedSurfaceView, SurfaceViewAdapter {
    constructor(context: Context) : super(context)
    constructor(context: Context, attrs: AttributeSet?) : super(context, attrs)

    private lateinit var leiaRendererAdapter: LeiaRendererAdapter

    // This setRenderer is called by the InterlacedSurfaceView during construction.
    override fun setRenderer(renderer: Renderer) {
        // wrap the InterlacedRenderer instance which the parent class constructed
        leiaRendererAdapter = LeiaRendererAdapter(renderer).also {
            super.setRenderer(it)
            setViewAsset(it.asset)
        }
    }

    // This setRenderer is called by the application.
    // The renderer you pass it should draw a 2x1 stereoscopic image, flipped upside down.
    override fun setRenderer(renderer: com.simongellis.vvb.emulator.Renderer) {
        leiaRendererAdapter.innerRenderer = renderer
    }
}