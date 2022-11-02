package com.simongellis.vvb.game

import com.simongellis.vvb.emulator.Renderer
import java.util.concurrent.locks.ReentrantReadWriteLock
import javax.microedition.khronos.egl.EGLConfig
import javax.microedition.khronos.opengles.GL10
import kotlin.concurrent.read
import kotlin.concurrent.write

class RendererWrapper(private var _renderer: Renderer) : Renderer {
    private val _lock = ReentrantReadWriteLock()
    private var _shouldInit = false
    private var _lastWidth: Int = 0
    private var _lastHeight: Int = 0

    override fun destroy() {
        _lock.write { _renderer.destroy() }
    }

    override fun onSurfaceCreated(gl: GL10?, config: EGLConfig?) {
        _lock.read { _renderer.onSurfaceCreated(gl, config) }
    }

    override fun onSurfaceChanged(gl: GL10?, width: Int, height: Int) {
        _lock.read {
            _lastWidth = width
            _lastHeight = height
            _shouldInit = false
            _renderer.onSurfaceChanged(gl, width, height)
        }
    }

    override fun onDrawFrame(gl: GL10?) {
        _lock.read {
            if (_shouldInit) {
                synchronized(this) {
                    if (_shouldInit) {
                        _shouldInit = false
                        _renderer.onSurfaceCreated(gl, null)
                        _renderer.onSurfaceChanged(gl, _lastWidth, _lastHeight)
                    }
                }
            }
            _renderer.onDrawFrame(gl)
        }
    }

    fun swapRenderer(newRenderer: Renderer) {
        _shouldInit = true
        val oldRenderer = _lock.write {
            val oldRenderer = _renderer
            _renderer = newRenderer
            oldRenderer
        }
        oldRenderer.destroy()
    }
}