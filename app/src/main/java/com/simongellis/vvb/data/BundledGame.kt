package com.simongellis.vvb.data

import android.net.Uri

data class BundledGame(
    val id: String,
    val name: String,
    val uri: Uri,
    val authors: List<String>,
)