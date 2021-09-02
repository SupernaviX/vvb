package com.simongellis.vvb.game

import android.app.Application
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.view.KeyEvent
import android.view.MotionEvent
import androidx.annotation.DrawableRes
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import com.simongellis.vvb.R
import com.simongellis.vvb.VvbApplication
import com.simongellis.vvb.emulator.Emulator
import com.simongellis.vvb.emulator.Input

class GameViewModel(application: Application): AndroidViewModel(application) {
    private val _application = getApplication<VvbApplication>()
    private val _emulator = Emulator.instance
    private val _inputBindingMapper = InputBindingMapper(viewModelScope, _application)

    fun getBoundInput(event: KeyEvent): Input? {
        return _inputBindingMapper.getBoundInput(event)
    }

    fun getAxisInputs(event: MotionEvent): Pair<List<Input>, List<Input>> {
        return _inputBindingMapper.getAxisInputs(event)
    }

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