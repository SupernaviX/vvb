package com.simongellis.vvb

import android.content.pm.ActivityInfo
import android.opengl.GLSurfaceView
import androidx.appcompat.app.AppCompatActivity
import android.os.Bundle
import android.util.Log
import kotlinx.android.synthetic.main.activity_main.*
import javax.microedition.khronos.egl.EGLConfig
import javax.microedition.khronos.opengles.GL10

const val TAG = "MainActivity";

@Suppress("unused")
class MainActivity : AppCompatActivity() {
    private var _rendererPtr = 0L

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        nativeOnCreate()
        requestedOrientation = ActivityInfo.SCREEN_ORIENTATION_LANDSCAPE
        setContentView(R.layout.activity_main)

        surface_view.setEGLContextClientVersion(2)
        surface_view.setRenderer(Renderer())
        surface_view.renderMode = GLSurfaceView.RENDERMODE_CONTINUOUSLY
    }

    override fun onDestroy() {
        super.onDestroy()
        nativeOnDestroy()
    }

    inner class Renderer : GLSurfaceView.Renderer {
        override fun onSurfaceCreated(gl: GL10?, config: EGLConfig?) {
            Log.d(TAG, "onSurfaceCreated start")
            nativeOnSurfaceCreated()
            Log.d(TAG, "onSurfaceCreated end")
        }

        override fun onSurfaceChanged(gl: GL10?, width: Int, height: Int) {
            Log.d(TAG, "onSurfaceChanged start")
            nativeOnSurfaceChanged(width, height)
            Log.d(TAG, "onSurfaceChanged end")
        }

        override fun onDrawFrame(gl: GL10?) {
            nativeOnDrawFrame()
        }
    }

    private external fun nativeOnCreate()
    private external fun nativeOnDestroy()
    private external fun nativeOnSurfaceCreated()
    private external fun nativeOnSurfaceChanged(width: Int, height: Int)
    private external fun nativeOnDrawFrame()

    companion object {
        init {
            System.loadLibrary("vvb")
        }
    }
}