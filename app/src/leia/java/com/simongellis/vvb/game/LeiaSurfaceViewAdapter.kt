package com.simongellis.vvb.game

import android.content.Context
import android.graphics.SurfaceTexture
import android.opengl.GLES20
import android.util.AttributeSet
import android.util.Log
import com.leia.sdk.LeiaSDK
import com.leia.sdk.views.InputViewsAsset
import com.leia.sdk.views.InterlacedSurfaceView
import javax.microedition.khronos.egl.EGLConfig
import javax.microedition.khronos.opengles.GL10

class LeiaSurfaceViewAdapter : InterlacedSurfaceView, SurfaceViewAdapter, LeiaSDK.Delegate {
    constructor(context: Context) : super(context)
    constructor(context: Context, attrs: AttributeSet?) : super(context, attrs)

    private var inner: com.simongellis.vvb.emulator.Renderer? = null

    init {
        val initArgs = LeiaSDK.InitArgs()
        initArgs.delegate = this
        initArgs.platform.context = context.applicationContext
        initArgs.platform.activity = getActivity(context)
        initArgs.enableFaceTracking = true
        LeiaSDK.createSDK(initArgs)
    }

    override fun setRenderer(renderer: Renderer) {
        val adapter = RendererAdapter(renderer)
        super.setRenderer(adapter)
        setViewAsset(adapter.asset)
    }

    override fun setRenderer(renderer: com.simongellis.vvb.emulator.Renderer) {
        inner = renderer
    }

    inner class RendererAdapter(private val interlacer: Renderer) : Renderer {
        val asset = InputViewsAsset()
        private var framebuffer: Int? = null
        private var textureId: Int? = null
        private var texture: SurfaceTexture? = null
        private var newSize: Pair<Int, Int>? = null

        init {
            asset.CreateEmptySurfaceForPicture(768, 224) {
                texture = it
                textureId = asset.GetSurfaceId()
            }
        }

        override fun onSurfaceCreated(gl: GL10?, p1: EGLConfig?) {
            interlacer.onSurfaceCreated(gl, p1)
            inner?.onSurfaceCreated()
        }

        override fun onSurfaceChanged(gl: GL10?, width: Int, height: Int) {
            newSize = width to height
            texture?.setDefaultBufferSize(width, height)
            interlacer.onSurfaceChanged(gl, width, height)
            inner?.onSurfaceChanged(width, height)
        }

        override fun onDrawFrame(gl: GL10) {
            ensureInitialized()
            framebuffer?.also {
                GLES20.glBindFramebuffer(GLES20.GL_FRAMEBUFFER, it)
                inner?.onDrawFrame()
            }
            interlacer.onDrawFrame(gl)
        }

        private fun ensureInitialized() {
            val (width, height) = newSize ?: return
            val textureId = this.textureId ?: return

            Log.i("LeiaSurfaceViewAdapter", "width: $width height: $height texture: $textureId")

            // TODO: clean up old framebuffer
            Log.i("LeiaSurfaceViewAdapter", "setting up $textureId (error 0x${GLES20.glGetError().toString(16)})")
            GLES20.glBindTexture(GLES20.GL_TEXTURE_2D, textureId)
            Log.i("LeiaSurfaceViewAdapter", "bound $textureId (error 0x${GLES20.glGetError().toString(16)})")
            GLES20.glTexImage2D(GLES20.GL_TEXTURE_2D, 0, GLES20.GL_RGB, width, height, 0, GLES20.GL_RGB, GLES20.GL_UNSIGNED_BYTE, null)
            Log.i("LeiaSurfaceViewAdapter", "initialized $textureId (error 0x${GLES20.glGetError().toString(16)})")

            val framebuffers = IntArray(1)
            GLES20.glGenFramebuffers(1, framebuffers, 0)
            val framebuffer = framebuffers[0]
            this.framebuffer = framebuffer
            Log.i("LeiaSurfaceViewAdapter", "initialized FB $framebuffer (error 0x${GLES20.glGetError().toString(16)})")
            GLES20.glBindFramebuffer(GLES20.GL_FRAMEBUFFER, framebuffer)
            Log.i("LeiaSurfaceViewAdapter", "bound FB $framebuffer (error 0x${GLES20.glGetError().toString(16)})")
            GLES20.glFramebufferTexture2D(GLES20.GL_FRAMEBUFFER, GLES20.GL_COLOR_ATTACHMENT0, GLES20.GL_TEXTURE_2D, textureId, 0)
            Log.i("LeiaSurfaceViewAdapter", "set texture of FB (error 0x${GLES20.glGetError().toString(16)})")
            GLES20.glBindFramebuffer(GLES20.GL_FRAMEBUFFER, 0)
            Log.i("LeiaSurfaceViewAdapter", "unbound FB (error 0x${GLES20.glGetError().toString(16)})")

            newSize = null
        }
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