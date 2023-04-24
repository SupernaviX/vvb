package com.simongellis.vvb.game

import android.content.Context
import android.opengl.GLSurfaceView
import android.util.AttributeSet
import javax.microedition.khronos.egl.EGLConfig
import javax.microedition.khronos.opengles.GL10

class GLSurfaceViewAdapter: GLSurfaceView, SurfaceViewAdapter {
    constructor(context: Context): super(context)
    constructor(context: Context, attrs: AttributeSet?): super(context, attrs)

    override fun setRenderer(renderer: com.simongellis.vvb.emulator.Renderer) {
        setEGLContextClientVersion(2)
        setRenderer(RendererAdapter(renderer))
        renderMode = RENDERMODE_CONTINUOUSLY
    }

    class RendererAdapter(private val inner: com.simongellis.vvb.emulator.Renderer): Renderer {
        override fun onSurfaceCreated(gl: GL10?, p1: EGLConfig?) {
            inner.onSurfaceCreated()
        }

        override fun onSurfaceChanged(gl: GL10?, width: Int, height: Int) {
            inner.onSurfaceChanged(width, height)
        }

        override fun onDrawFrame(gl: GL10?) {
            inner.onDrawFrame()
        }
    }
}