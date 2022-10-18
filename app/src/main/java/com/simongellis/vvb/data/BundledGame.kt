package com.simongellis.vvb.data

import android.net.Uri
import kotlinx.serialization.Serializable

@Serializable
data class BundledGame(
    val id: String,
    val name: String,
    @Serializable(with = UriSerializer::class)
    val uri: Uri,
    val authors: List<String>,
)