package com.simongellis.vvb.game

import android.app.Application
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import androidx.annotation.DrawableRes
import androidx.lifecycle.AndroidViewModel
import com.simongellis.vvb.R
import com.simongellis.vvb.VvbApplication
import com.simongellis.vvb.emulator.Emulator

class GameViewModel(application: Application): AndroidViewModel(application) {
    private val _application = getApplication<VvbApplication>()
    private val _emulator = Emulator.instance

    fun pauseGame() {
        _emulator.pause()
    }

    fun resumeGame() {
        _emulator.resume()
    }

    fun loadPreviewImage() {
        _emulator.loadImage(
            loadBitmap(R.drawable.vbtitlescreen_left),
            loadBitmap(R.drawable.vbtitlescreen_right)
        )
    }

    private fun loadBitmap(@DrawableRes id: Int): Bitmap {
        val options = BitmapFactory.Options().apply { inScaled = false }
        return BitmapFactory.decodeResource(_application.resources, id, options)
    }
}