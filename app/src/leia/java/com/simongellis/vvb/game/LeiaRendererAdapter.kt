package com.simongellis.vvb.game

import android.graphics.SurfaceTexture
import android.opengl.GLES20.*
import com.leia.android.opengl.LeiaGLSurfaceView
import com.leia.sdk.views.InputViewsAsset
import com.simongellis.vvb.emulator.Renderer
import javax.microedition.khronos.egl.EGLConfig
import javax.microedition.khronos.opengles.GL10

class LeiaRendererAdapter(private val leiaRenderer: LeiaGLSurfaceView.Renderer) :
    LeiaGLSurfaceView.Renderer {
    val asset = InputViewsAsset()
    lateinit var innerRenderer: Renderer

    private var stale = true
    private var framebuffer: Int? = null
    private var size: Pair<Int, Int>? = null
    private var surfaceTexture: SurfaceTexture? = null
    private var surfaceTextureId: Int? = null

    override fun onSurfaceCreated(gl: GL10, config: EGLConfig) {
        // Set the surface width/height to anything nonzero.
        // We will get the real size in onSurfaceChanged before they're used for real.
        val (width, height) = size ?: (768 to 224)
        updateSurfaceSize(width, height)

        // create new framebuffer
        val fbIds = IntArray(1)
        glGenFramebuffers(1, fbIds, 0)
        framebuffer = fbIds[0]

        innerRenderer.onSurfaceCreated()
        leiaRenderer.onSurfaceCreated(gl, config)
    }


    override fun onSurfaceChanged(gl: GL10, width: Int, height: Int) {
        size = width to height
        updateSurfaceSize(width, height)

        innerRenderer.onSurfaceChanged(width, height)
        leiaRenderer.onSurfaceChanged(gl, width, height)
    }

    override fun onDrawFrame(gl: GL10) {
        getPreparedFramebuffer()?.let {
            // innerRenderer.onDrawFrame should draw an upside-down 2x1 image to this FB.
            glBindFramebuffer(GL_FRAMEBUFFER, it)
            innerRenderer.onDrawFrame()
        }
        leiaRenderer.onDrawFrame(gl)
    }

    private fun updateSurfaceSize(width: Int, height: Int) {
        val surface = surfaceTexture
        if (surface == null || !asset.IsSurfaceValid()) {
            // surface is no longer usable, recreate it
            surfaceTexture = null
            surfaceTextureId = null
            asset.CreateEmptySurfaceForPicture(width, height) {
                // NB: this block gets called during InterlacedRenderer.onDrawFrame
                // any time assets are recreated
                surfaceTexture = it
                surfaceTextureId = asset.GetSurfaceId()
                stale = true
            }
        } else {
            surface.setDefaultBufferSize(width, height)
        }

        stale = true
    }

    private fun getPreparedFramebuffer(): Int? {
        if (!stale) return framebuffer

        val framebuffer = framebuffer ?: return null
        val (width, height) = size ?: return null
        val textureId = surfaceTextureId ?: return null
        stale = false

        // Define the texture your app is rendering to.
        glBindTexture(GL_TEXTURE_2D, textureId)
        glTexImage2D(GL_TEXTURE_2D, 0, GL_RGB, width, height, 0, GL_RGB, GL_UNSIGNED_BYTE, null)

        // Attach the texture to our framebuffer.
        glBindFramebuffer(GL_FRAMEBUFFER, framebuffer)
        glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, textureId, 0)
        glBindFramebuffer(GL_FRAMEBUFFER, 0)

        return framebuffer
    }
}