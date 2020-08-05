package com.simongellis.vvb

import android.content.pm.ActivityInfo
import android.graphics.BitmapFactory
import android.opengl.GLSurfaceView
import androidx.appcompat.app.AppCompatActivity
import android.os.Bundle
import android.util.Log
import android.view.MenuItem
import android.view.View
import android.widget.PopupMenu
import com.google.android.gms.common.ConnectionResult
import com.google.android.gms.common.GoogleApiAvailability
import kotlinx.android.synthetic.main.activity_main.*
import java.nio.ByteBuffer
import javax.microedition.khronos.egl.EGLConfig
import javax.microedition.khronos.opengles.GL10

const val TAG = "MainActivity"

@Suppress("unused")
class MainActivity : AppCompatActivity(), PopupMenu.OnMenuItemClickListener {
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

    override fun onResume() {
        super.onResume()
        nativeOnResume()
    }

    override fun onDestroy() {
        super.onDestroy()
        nativeOnDestroy()
    }

    inner class Renderer : GLSurfaceView.Renderer {
        override fun onSurfaceCreated(gl: GL10?, config: EGLConfig?) {
            Log.d(TAG, "onSurfaceCreated start")
            nativeOnSurfaceCreated(loadTitleScreen())
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

    fun showSettings(view: View) {
        val popup = PopupMenu(this, view)
        popup.menuInflater.inflate(R.menu.settings_menu, popup.menu)
        popup.setOnMenuItemClickListener(this)
        popup.show()
    }

    override fun onMenuItemClick(item: MenuItem): Boolean {
        if (item.itemId == R.id.switch_viewer) {
            val play = GoogleApiAvailability.getInstance()
            val availability = play.isGooglePlayServicesAvailable(this)
            if (availability != ConnectionResult.SUCCESS) {
                play.getErrorDialog(this, availability, 420691337).show()
            } else {
                nativeSwitchViewer()
            }
            return true
        }
        return false
    }

    private fun loadTitleScreen(): ByteBuffer {
        val options = BitmapFactory.Options()
        options.inScaled = false
        val bmp = BitmapFactory.decodeResource(resources, R.drawable.vbtitlescreen, options)
        val buffer = ByteBuffer.allocateDirect(bmp.byteCount)
        bmp.copyPixelsToBuffer(buffer)
        buffer.rewind()
        return buffer
    }

    private external fun nativeOnCreate()
    private external fun nativeOnResume()
    private external fun nativeOnDestroy()
    private external fun nativeSwitchViewer()
    private external fun nativeOnSurfaceCreated(titleScreen: ByteBuffer)
    private external fun nativeOnSurfaceChanged(width: Int, height: Int)
    private external fun nativeOnDrawFrame()

    companion object {
        init {
            System.loadLibrary("vvb")
        }
    }
}