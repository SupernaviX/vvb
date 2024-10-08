package com.simongellis.vvb.data

import android.content.Context
import android.net.Uri
import android.provider.OpenableColumns
import com.simongellis.vvb.emulator.GamePak
import com.simongellis.vvb.emulator.StateSlot
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import java.util.*

class GameRepository(scope: CoroutineScope, val context: Context) {
    private val _dao = PreferencesDao.forClass<GameData>(context)
    private val _fileWatcher = FileWatcher(scope)
    private val _filenames = HashMap<Uri, String?>()

    val recentGames by lazy {
        _dao.watchAll().map { games ->
            games
                .sortedByDescending { it.lastPlayed }
                .take(10)
                .mapNotNull(::fromData)
        }
    }

    fun hasRecentGames(): Boolean {
        return _dao.getAll()
            .sortedByDescending { it.lastPlayed }
            .take(10)
            .mapNotNull(::fromData)
            .isNotEmpty()
    }

    fun getGameData(id: String, uri: Uri): GameData {
        return _dao.get(id) ?: GameData(id, uri, Date(), 0, true)
    }

    fun watchGame(id: String) = _dao.watch(id).map { fromData(it) }

    fun watchStateSlots(gamePak: GamePak): Flow<List<StateSlot>> {
        val flows = (0..9).map { index -> watchStateSlot(gamePak.getStateSlot(index)) }
        return combine(flows) { it.toList() }
    }

    fun markAsPlayed(id: String, uri: Uri) {
        val data = _dao.get(id) ?: GameData(id, uri, Date(), 0, true)
        val newData = data.copy(uri = uri, lastPlayed = Date())
        _dao.put(newData)
    }

    fun selectStateSlot(id: String, slot: Int) {
        val data = _dao.get(id) ?: return
        val newData = data.copy(stateSlot = slot)
        _dao.put(newData)
    }

    fun setAutoSaveEnabled(id: String, enabled: Boolean) {
        val data = _dao.get(id) ?: return
        val newData = data.copy(autoSaveEnabled = enabled)
        _dao.put(newData)
    }

    private fun fromData(data: GameData): Game? {
        val name = getName(data.uri)
        return name?.let {
            Game(data.id, it, data.uri, data.lastPlayed, data.stateSlot, data.autoSaveEnabled)
        }
    }

    private fun getName(uri: Uri): String? {
        val filename = getFilename(uri)
        return filename?.substringBeforeLast('.')
    }

    private fun getFilename(uri: Uri) = _filenames.getOrPut(uri) {
        val cursor = try {
            context.contentResolver.query(
                uri,
                arrayOf(OpenableColumns.DISPLAY_NAME),
                null,
                null,
                null
            )
        } catch (ex: Exception) {
            return null
        }
        cursor?.use {
            if (it.moveToFirst()) {
                return it.getString(it.getColumnIndexOrThrow(OpenableColumns.DISPLAY_NAME))
            }
        }
        return uri.lastPathSegment?.substringAfterLast('/')
    }

    private fun watchStateSlot(slot: StateSlot): Flow<StateSlot> {
        return _fileWatcher.watch(slot.file).map { StateSlot(it, slot.name) }
    }
}