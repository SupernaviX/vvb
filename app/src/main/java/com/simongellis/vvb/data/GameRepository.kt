package com.simongellis.vvb.data

import android.content.Context
import android.net.Uri
import android.provider.OpenableColumns
import kotlinx.coroutines.flow.map
import java.util.*

class GameRepository(val context: Context) {
    private val dao = PreferencesDao.forClass(GameData.serializer(), context)

    val recentGames by lazy {
        dao.watchAll().map { games ->
            games
                .sortedByDescending { it.lastPlayed }
                .take(10)
                .map(::fromData)
        }
    }

    fun getGame(uri: Uri): Game {
        val data = GameData(uri, Date())
        return fromData(data)
    }

    fun markAsPlayed(game: Game) {
        val data = dao.get(game.id) ?: toData(game)
        val newData = data.copy(lastPlayed = Date())
        dao.put(newData)
    }

    private fun fromData(data: GameData): Game {
        return Game(data.id, getName(data.uri), data.uri, data.lastPlayed)
    }

    private fun toData(game: Game): GameData {
        return GameData(game.uri, game.lastPlayed)
    }

    private fun getName(uri: Uri): String {
        val filename = getFilename(uri)
        val sep = filename.lastIndexOf('.')
        if (sep == -1) {
            return filename
        }
        return filename.substring(0, sep)
    }

    private fun getFilename(uri: Uri): String {
        val cursor = context.contentResolver.query(uri, arrayOf(OpenableColumns.DISPLAY_NAME), null, null, null)
        cursor?.use {
            if (it.moveToFirst()) {
                return it.getString(it.getColumnIndex(OpenableColumns.DISPLAY_NAME))
            }
        }
        return uri.lastPathSegment!!.substringAfterLast('/')
    }
}