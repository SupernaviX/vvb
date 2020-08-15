package com.simongellis.vvb

import android.app.Activity
import android.content.Intent
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
    private lateinit var _emulator: Emulator
    private lateinit var _renderer: Renderer

    val GAME_CHOSEN = 2

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        nativeInitialize()
        _emulator = Emulator(applicationContext)
        _renderer = Renderer(_emulator)

        requestedOrientation = ActivityInfo.SCREEN_ORIENTATION_LANDSCAPE
        setContentView(R.layout.activity_main)

        surface_view.setEGLContextClientVersion(2)
        surface_view.setRenderer(_renderer)
        surface_view.renderMode = GLSurfaceView.RENDERMODE_CONTINUOUSLY

        _emulator.loadImage()
    }

    override fun onPause() {
        super.onPause()
        surface_view.onPause()
    }

    override fun onResume() {
        super.onResume()
        surface_view.onResume()
        _renderer.ensureDeviceParams()
        _emulator.loadImage()
    }

    override fun onDestroy() {
        super.onDestroy()
        _renderer.destroy()
        _emulator.destroy()
    }

    fun showSettings(view: View) {
        val popup = PopupMenu(this, view)
        popup.menuInflater.inflate(R.menu.settings_menu, popup.menu)
        popup.setOnMenuItemClickListener(this)
        popup.show()
    }

    override fun onMenuItemClick(item: MenuItem): Boolean {
        if (item.itemId == R.id.load_game) {
            val intent = Intent(Intent.ACTION_OPEN_DOCUMENT).apply {
                addCategory(Intent.CATEGORY_OPENABLE)
                type = "*/*"
            }
            startActivityForResult(intent, GAME_CHOSEN)
        }
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

    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)
        if (requestCode == GAME_CHOSEN && resultCode == Activity.RESULT_OK) {
            data?.data?.also { uri ->
                _emulator.loadGamePak(uri)
                _emulator.run()
            }
        }
    }

    private external fun nativeInitialize()

    companion object {
        init {
            System.loadLibrary("vvb")
        }
    }
}