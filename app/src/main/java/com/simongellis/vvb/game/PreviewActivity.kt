package com.simongellis.vvb.game

import android.os.Bundle
import androidx.appcompat.app.AppCompatActivity
import com.simongellis.vvb.emulator.Emulator

class PreviewActivity: AppCompatActivity() {
    private lateinit var _view: GameView
    private lateinit var _emulator: Emulator

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        _view = GameView(baseContext)
        requestedOrientation = _view.requestedOrientation
        setContentView(_view)

        _emulator = Emulator.getInstance()
        _emulator.loadImage(baseContext)
    }

    override fun onPause() {
        super.onPause()
        _view.onPause()
    }

    override fun onResume() {
        super.onResume()
        _view.onResume()
        _emulator.loadImage(baseContext)
    }
}