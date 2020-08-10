package com.simongellis.vvb

import android.content.pm.ActivityInfo
import android.opengl.GLSurfaceView
import androidx.appcompat.app.AppCompatActivity
import android.os.Bundle
import android.view.MenuItem
import android.view.View
import android.widget.PopupMenu
import com.google.android.gms.common.ConnectionResult
import com.google.android.gms.common.GoogleApiAvailability
import kotlinx.android.synthetic.main.activity_main.*

class MainActivity : AppCompatActivity(), PopupMenu.OnMenuItemClickListener {
    private lateinit var _renderer: Renderer

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        nativeInitialize()
        _renderer = Renderer(resources)
        requestedOrientation = ActivityInfo.SCREEN_ORIENTATION_LANDSCAPE
        setContentView(R.layout.activity_main)

        surface_view.setEGLContextClientVersion(2)
        surface_view.setRenderer(_renderer)
        surface_view.renderMode = GLSurfaceView.RENDERMODE_CONTINUOUSLY
    }

    override fun onPause() {
        super.onPause()
        surface_view.onPause()
    }

    override fun onResume() {
        super.onResume()
        surface_view.onResume()
        _renderer.ensureDeviceParams()
    }

    override fun onDestroy() {
        super.onDestroy()
        _renderer.destroy()
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
                _renderer.changeDeviceParams()
            }
            return true
        }
        return false
    }

    private external fun nativeInitialize()

    companion object {
        init {
            System.loadLibrary("vvb")
        }
    }
}