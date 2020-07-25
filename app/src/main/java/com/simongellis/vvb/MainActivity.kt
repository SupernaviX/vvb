package com.simongellis.vvb

import androidx.appcompat.app.AppCompatActivity
import android.os.Bundle
import kotlinx.android.synthetic.main.activity_main.*

class MainActivity : AppCompatActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        sample_text.text = stringFromRust()
    }

    private external fun stringFromRust(): String

    companion object {
        init {
            System.loadLibrary("vvb")
        }
    }
}