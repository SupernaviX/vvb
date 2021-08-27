package com.simongellis.vvb.game

import android.os.Bundle
import androidx.activity.viewModels
import androidx.appcompat.app.AppCompatActivity
import com.simongellis.vvb.emulator.Emulator
import com.simongellis.vvb.emulator.VvbLibrary

class PreviewActivity: AppCompatActivity() {
    private val viewModel: GameViewModel by viewModels()

    private lateinit var _view: GameView
    private lateinit var _preferences: GamePreferences

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        VvbLibrary.instance.initialize(this)

        _view = GameView(baseContext)
        _preferences = GamePreferences(baseContext)
        requestedOrientation = _view.requestedOrientation
        setContentView(_view)

        viewModel.loadPreviewImage()
    }

    override fun onPause() {
        super.onPause()
        _view.onPause()
    }

    override fun onResume() {
        super.onResume()
        _view.onResume()
        viewModel.loadPreviewImage()
    }
}