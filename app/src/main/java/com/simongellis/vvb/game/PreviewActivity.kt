package com.simongellis.vvb.game

import android.content.pm.ActivityInfo
import android.os.Bundle
import androidx.appcompat.app.AppCompatActivity
import com.simongellis.vvb.emulator.Emulator

class PreviewActivity: AppCompatActivity() {
    private lateinit var _view: GameView
    private lateinit var _emulator: Emulator

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        _view = GameView(applicationContext)
        requestedOrientation = _view.requestedOrientation
        setContentView(_view)

        _emulator = Emulator.getInstance()
        _emulator.loadImage(applicationContext)
    }

    override fun onPause() {
        super.onPause()
        _view.onPause()
    }

    override fun onResume() {
        super.onResume()
        _view.onResume()
        _emulator.loadImage(applicationContext)
    }
}