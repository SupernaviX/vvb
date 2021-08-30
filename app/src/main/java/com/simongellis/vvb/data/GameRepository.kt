package com.simongellis.vvb.data

import android.content.Context
import android.net.Uri
import android.provider.OpenableColumns
import kotlinx.coroutines.flow.map
import java.io.File
import java.util.*
import kotlin.collections.HashMap

class GameRepository(val context: Context) {
    private val _dao = PreferencesDao.forClass(GameData.serializer(), context)
    private val _filenames = HashMap<Uri, String>()
    private val _saveStates = HashMap<String, File>()

    val recentGames by lazy {
        _dao.watchAll().map { games ->
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

    fun markAsPlayed(id: String, uri: Uri) {
        val data = _dao.get(id) ?: GameData(uri, Date())
        val newData = data.copy(uri = uri, lastPlayed = Date())
        _dao.put(newData)
    }

    private fun fromData(data: GameData): Game {
        return Game(data.id, getName(data.uri), data.uri, data.lastPlayed, getSaveState(data.id))
    }

    private fun getName(uri: Uri): String {
        val filename = getFilename(uri)
        return filename.substringBeforeLast('.')
    }

    private fun getFilename(uri: Uri) = _filenames.getOrPut(uri) {
        val cursor = context.contentResolver.query(
            uri,
            arrayOf(OpenableColumns.DISPLAY_NAME),
            null,
            null,
            null
        )
        cursor?.use {
            if (it.moveToFirst()) {
                return it.getString(it.getColumnIndex(OpenableColumns.DISPLAY_NAME))
            }
        }
        return uri.lastPathSegment!!.substringAfterLast('/')
    }

    private fun getSaveState(id: String) = _saveStates.getOrPut(id) {
        val saveStateDir = File(context.filesDir, id)
        saveStateDir.mkdir()
        File(saveStateDir, "0.sav")
    }
}