package com.simongellis.vvb.data

import android.net.Uri
import com.simongellis.vvb.game.GamePakLoader
import kotlinx.serialization.Serializable
import java.util.*

@Serializable
data class Game(
    @Serializable(with = UriSerializer::class)
    val uri: Uri,
    @Serializable(with = DateSerializer::class)
    val lastPlayed: Date
): Entity {
    override val id = uri.toString()
    val name = GamePakLoader.getName(uri)
}