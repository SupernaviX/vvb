package com.simongellis.vvb.data

import android.content.Context
import android.net.Uri
import android.provider.OpenableColumns
import kotlinx.coroutines.flow.*
import java.util.*
import kotlin.collections.HashMap

class GameRepository(val context: Context) {
    private val _dao = PreferencesDao.forClass(GameData.serializer(), context)
    private val _fileDao = FileDao(context)
    private val _filenames = HashMap<Uri, String>()

    val recentGames by lazy {
        _dao.watchAll().map { games ->
            games
                .sortedByDescending { it.lastPlayed }
                .take(10)
                .map { fromData(it, getState(it)) }
        }
    }

    fun getGame(uri: Uri): Game {
        val data = GameData(uri, Date())
        return fromData(data, getState(data))
    }

    fun watchGame(id: String): Flow<Game> {
        val dataFlow = _dao.watch(id)
        val stateFlow = dataFlow.flatMapLatest { watchState(it) }
        return dataFlow.combine(stateFlow, ::fromData)
    }

    fun markAsPlayed(id: String, uri: Uri) {
        val data = _dao.get(id) ?: GameData(uri, Date())
        val newData = data.copy(uri = uri, lastPlayed = Date())
        _dao.put(newData)
    }

    private fun fromData(data: GameData, currentState: SaveState): Game {
        return Game(data.id, getName(data.uri), data.uri, data.lastPlayed, currentState)
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

    private fun getState(data: GameData)
        = SaveState(_fileDao.get(getStatePath(data)))
    private fun watchState(data: GameData)
        = _fileDao.watch(getStatePath(data)).map{ SaveState(it) }
    private fun getStatePath(data: GameData)
        = "${data.id}/save_states/0.sav"
}