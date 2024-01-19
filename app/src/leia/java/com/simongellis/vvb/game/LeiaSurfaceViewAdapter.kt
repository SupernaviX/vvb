package com.simongellis.vvb.game

import android.content.Context
import android.util.AttributeSet
import com.leia.sdk.views.InputViewsAsset
import com.leia.sdk.views.InterlacedSurfaceView

class LeiaSurfaceViewAdapter : InterlacedSurfaceView, SurfaceViewAdapter {
    constructor(context: Context) : super(context)
    constructor(context: Context, attrs: AttributeSet?) : super(context, attrs)

    override fun setRenderer(renderer: com.simongellis.vvb.emulator.Renderer) {
        setViewAsset(InputViewsAsset(RendererImpl(renderer)))
    }
}