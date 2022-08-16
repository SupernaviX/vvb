package com.simongellis.vvb.data

import android.content.Context
import com.simongellis.vvb.R
import kotlinx.serialization.json.Json
import kotlinx.serialization.serializer

class BundledGameRepository(context: Context) {
    val bundledGames = readBundledGames(context)

    private fun readBundledGames(context: Context): List<BundledGame> {
        val text = context.resources.openRawResource(R.raw.bundledgames).bufferedReader().use { it.readText() }
        val gameData: List<BundledGameData> = Json.decodeFromString(serializer(), text)
        return gameData.map { BundledGame(it.id, it.name, it.uri, it.authors) }
    }
}