package com.simongellis.vvb.data

import android.content.Context
import com.simongellis.vvb.R
import kotlinx.serialization.json.Json
import kotlinx.serialization.serializer

class BundledGameRepository(context: Context) {
    val bundledGames = readBundledGames(context)

    private fun readBundledGames(context: Context): List<BundledGame> {
        val text = context.resources.openRawResource(R.raw.bundledgames).bufferedReader().use { it.readText() }
        return Json.decodeFromString(serializer(), text)
    }
}