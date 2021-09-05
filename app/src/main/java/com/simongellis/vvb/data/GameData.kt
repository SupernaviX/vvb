package com.simongellis.vvb.data

import android.net.Uri
import kotlinx.serialization.Serializable
import java.util.*

@Serializable
data class GameData(
    @Serializable(with = UriSerializer::class)
    val uri: Uri,
    @Serializable(with = DateSerializer::class)
    val lastPlayed: Date
): Entity {
    override val id = uri.lastPathSegment!!.substringAfterLast('/')
}