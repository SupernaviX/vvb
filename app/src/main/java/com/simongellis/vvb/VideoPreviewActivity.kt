package com.simongellis.vvb

import android.content.pm.ActivityInfo
import android.os.Bundle
import androidx.appcompat.app.AppCompatActivity
import com.simongellis.vvb.databinding.ActivityGameBinding
import com.simongellis.vvb.emulator.Emulator

class VideoPreviewActivity: AppCompatActivity() {
    private lateinit var _binding: ActivityGameBinding
    private lateinit var _emulator: Emulator

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        _binding = ActivityGameBinding.inflate(layoutInflater)

        requestedOrientation = ActivityInfo.SCREEN_ORIENTATION_LANDSCAPE
        setContentView(_binding.root)

        _emulator = Emulator.getInstance()
        _emulator.loadImage(applicationContext)
    }

    override fun onPause() {
        super.onPause()
        _binding.gameView.onPause()
    }

    override fun onResume() {
        super.onResume()
        _binding.gameView.onResume()
        _emulator.loadImage(applicationContext)
    }
}