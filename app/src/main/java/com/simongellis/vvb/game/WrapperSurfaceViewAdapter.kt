package com.simongellis.vvb.game

import android.content.Context
import android.util.AttributeSet
import android.view.LayoutInflater
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

    @Suppress("TypeParameterFindViewById")
    private val inner: SurfaceViewAdapter
        get() = findViewById(R.id.inner_surface_view) as SurfaceViewAdapter

    override fun setRenderer(renderer: Renderer) {
        inner.setRenderer(renderer)
    }

    override fun onPause() {
        inner.onPause()
    }

    override fun onResume() {
        inner.onResume()
    }

}