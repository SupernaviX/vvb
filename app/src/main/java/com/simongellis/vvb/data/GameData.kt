package com.simongellis.vvb.data

import android.net.Uri
import kotlinx.serialization.Serializable
import java.util.*

@Serializable
data class GameData(
    override val id: String,
    @Serializable(with = UriSerializer::class)
    val uri: Uri,
    @Serializable(with = DateSerializer::class)
    val lastPlayed: Date,
    val stateSlot: Int,
    val autoSaveEnabled: Boolean,
): Entity