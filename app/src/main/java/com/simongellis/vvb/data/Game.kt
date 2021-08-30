package com.simongellis.vvb.data

import android.net.Uri
import java.io.File
import java.util.*

data class Game(
    val id: String,
    val name: String,
    val uri: Uri,
    val lastPlayed: Date,
    val currentState: File
)