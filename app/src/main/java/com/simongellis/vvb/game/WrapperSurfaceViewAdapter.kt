package com.simongellis.vvb.game

import android.content.Context
import android.util.AttributeSet
import android.view.LayoutInflater
import android.view.View
import android.widget.LinearLayout
import com.simongellis.vvb.R
import com.simongellis.vvb.databinding.WrapperSurfaceViewAdapterBinding
import com.simongellis.vvb.emulator.Renderer

class WrapperSurfaceViewAdapter : LinearLayout, SurfaceViewAdapter {
    constructor(context: Context): super(context)
    constructor(context: Context, attrs: AttributeSet?): super(context, attrs)

    init {
        val layoutInflater = LayoutInflater.from(context)
        WrapperSurfaceViewAdapterBinding.inflate(layoutInflater, this, true)
    }

    @Suppress("K1TypeParameterFindViewById")
    private val inner: SurfaceViewAdapter
        get() = findViewById(R.id.inner_surface_view) as SurfaceViewAdapter
    private val inner2d: GLSurfaceViewAdapter?
        get() = findViewWithTag("surface_view_2d")

    override fun setRenderer(renderer: Renderer) {
        inner.setRenderer(renderer)
        inner2d?.setRenderer(renderer)
        if (!renderer.isLeia && inner2d != null) {
            (inner as? View)?.visibility = GONE
            inner2d?.visibility = VISIBLE
        } else {
            (inner as? View)?.visibility = VISIBLE
            inner2d?.visibility = GONE
        }
    }

    override fun onPause() {
        inner.onPause()
    }

    override fun onResume() {
        inner.onResume()
    }

}