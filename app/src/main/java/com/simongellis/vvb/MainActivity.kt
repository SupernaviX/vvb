package com.simongellis.vvb

import android.content.pm.ActivityInfo
import android.opengl.GLSurfaceView
import androidx.appcompat.app.AppCompatActivity
import android.os.Bundle
import android.util.Log
import kotlinx.android.synthetic.main.activity_main.*
import javax.microedition.khronos.egl.EGLConfig
import javax.microedition.khronos.opengles.GL10

class MainActivity : AppCompatActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        requestedOrientation = ActivityInfo.SCREEN_ORIENTATION_LANDSCAPE
        setContentView(R.layout.activity_main)

        surface_view.setEGLContextClientVersion(2)
        surface_view.setRenderer(Renderer())
        surface_view.renderMode = GLSurfaceView.RENDERMODE_CONTINUOUSLY

        Log.d("MainActivity", stringFromRust())
    }

    inner class Renderer : GLSurfaceView.Renderer {
        override fun onSurfaceCreated(gl: GL10?, config: EGLConfig?) {
            // TODO("Not yet implemented")
        }

        override fun onSurfaceChanged(gl: GL10?, width: Int, height: Int) {
            // TODO("Not yet implemented")
        }

        override fun onDrawFrame(gl: GL10?) {
            // TODO("Not yet implemented")
        }
    }

    private external fun stringFromRust(): String

    companion object {
        init {
            System.loadLibrary("vvb")
        }
    }
}