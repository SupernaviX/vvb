package com.simongellis.vvb.game

import android.graphics.SurfaceTexture
import android.opengl.GLES20.*
import android.opengl.GLES30
import android.os.Handler
import android.os.Looper
import android.util.Size
import com.leia.sdk.graphics.Interlacer
import com.leia.sdk.graphics.SurfaceTextureReadyCallback
import com.leia.sdk.views.InputGLBinding
import com.leia.sdk.views.InputViewsAsset
import com.leia.sdk.views.InterlacedRenderer
import com.simongellis.vvb.emulator.Renderer

/**
 *  An InputViewsAsset.Impl which uses an OpenGL "Renderer" instance to draw to the screen.
 *  The renderer should draw a 2x1 side-by-side image _upside down_.
 */
class RendererImpl(val renderer: Renderer, listener: SurfaceTextureReadyCallback?) : InputViewsAsset.Impl(listener) {
    constructor(renderer: Renderer) : this(renderer, null)

    override fun createGLBinding(): InputGLBinding {
        return RendererGLBinding(this, this.mSurfaceTextureReadyListener)
    }

    class RendererGLBinding(impl: RendererImpl, private val _listener: SurfaceTextureReadyCallback?) : InputGLBinding(impl) {
        private val _texture = Texture()
        private var _surface: SurfaceTexture? = null

        private var _stale = true
        private var _framebuffer: Int? = null
        private var _size: Size? = null

        override fun reset() {
            destroyFramebuffer()
            destroyTexture()

            _surface?.release()
            _surface = null

            _stale = true
            _size = null
            super.reset()
        }

        override fun update(interlacer: InterlacedRenderer, isProtected: Boolean) {
            super.update(interlacer, isProtected)
            if (_surface?.isReleased == true) {
                _surface = null
            }
            if (this.mIsValid) {
                val asset = updateAsset(RendererImpl::class.java) ?: return
                if (_texture.glId == -1) {
                    initTexture(isProtected)
                    initFramebuffer()
                    _stale = true

                    _surface = SurfaceTexture(_texture.glId)
                    _listener?.also {
                        Handler(Looper.getMainLooper()).post { it.onSurfaceTextureReady(_surface) }
                    }
                    asset.renderer.onSurfaceCreated()
                }
            }
        }

        override fun render(interlacer: Interlacer, viewportWidth: Int, viewportHeight: Int) {
            val asset = updateAsset(RendererImpl::class.java) ?: return
            if (this._texture.glId == -1) { return }

            val newSize = Size(viewportWidth, viewportHeight)
            if (_size != newSize) {
                _surface!!.setDefaultBufferSize(viewportWidth, viewportHeight)
                asset.renderer.onSurfaceChanged(viewportWidth, viewportHeight)
                _size = newSize
                _stale = true
            }

            getPreparedFramebuffer()?.let {
                // renderer.onDrawFrame should draw an upside-down 2x1 image to this FB.
                glBindFramebuffer(GL_FRAMEBUFFER, it)
                asset.renderer.onDrawFrame()
            }
            interlacer.doPostProcess(viewportWidth, viewportHeight, this._texture.glId, this._texture.glType)
        }

        private fun initTexture(isProtected: Boolean) {
            val textureIds = IntArray(1)
            glGenTextures(textureIds.size, textureIds, 0)

            glBindTexture(GL_TEXTURE_2D, textureIds[0])
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR)
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR)
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE)
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE)
            if (isProtected) {
                GLES30.glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_PROTECTED_EXT, 1)
            }

            _texture.glId = textureIds[0]
            _texture.glType = GL_TEXTURE_2D
        }

        private fun destroyTexture() {
            if (_texture.glId != -1 && isValidGLContext) {
                val textureIds = intArrayOf(_texture.glId)
                glDeleteTextures(textureIds.size, textureIds, 0)
            }
            _texture.glId = -1
            _texture.glType = -1
        }

        private fun initFramebuffer() {
            val fbIds = IntArray(1)
            glGenFramebuffers(fbIds.size, fbIds, 0)
            _framebuffer = fbIds[0]
        }


        private fun destroyFramebuffer() {
            val framebuffer = _framebuffer ?: return
            val fbIds = intArrayOf(framebuffer)
            glDeleteFramebuffers(fbIds.size, fbIds, 0)
            _framebuffer = null
        }

        private fun getPreparedFramebuffer(): Int? {
            if (!_stale) return _framebuffer

            val framebuffer = _framebuffer ?: return null
            val size = _size ?: return null
            val textureId = if (_texture.glId != -1) { _texture.glId } else { return null }
            _stale = false

            // Define the texture your app is rendering to.
            glBindTexture(GL_TEXTURE_2D, textureId)
            glTexImage2D(GL_TEXTURE_2D, 0, GL_RGB, size.width, size.height, 0, GL_RGB, GL_UNSIGNED_BYTE, null)

            // Attach the texture to our framebuffer.
            glBindFramebuffer(GL_FRAMEBUFFER, framebuffer)
            glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, textureId, 0)
            glBindFramebuffer(GL_FRAMEBUFFER, 0)

            return framebuffer
        }
    }
}