package com.simongellis.vvb

import android.app.Application
import android.net.Uri
import android.util.Log
import android.widget.Toast
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import com.simongellis.vvb.data.GameRepository
import com.simongellis.vvb.emulator.Emulator
import com.simongellis.vvb.game.GamePakLoader
import kotlinx.coroutines.flow.*
import org.acra.ACRA
import java.lang.Exception

class MainViewModel(application: Application): AndroidViewModel(application) {
    private val _gameRepo = GameRepository(viewModelScope, application)
    private val _application = getApplication<VvbApplication>()
    private val _emulator = Emulator.instance
    private val _gamePakLoader = GamePakLoader(application)

    private val _loadedGameId = MutableStateFlow<String?>(null)
    val loadedGame = forCurrentGame { _gameRepo.watchGame(it) }
    val stateSlots = forCurrentGame { _gameRepo.watchStateSlots(it) }
    val currentStateSlot = loadedGame.combine(stateSlots) { game, states ->
        game?.let { states?.get(it.stateSlot) }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    enum class GameEvent {
        Opened,
        Closed
    }
    val lastEvent = MutableStateFlow<GameEvent?>(null)

    fun loadGame(uri: Uri): Boolean {
        return try {
            val game = _gameRepo.getGame(uri)
            val gamePak = _gamePakLoader.tryLoad(game.id, uri)
            val autoSave = _gameRepo.getAutoSave(game.id)

            _emulator.loadGamePak(gamePak)
            if (game.autoSaveEnabled) {
                _emulator.setAutoSaveFile(autoSave.file)
                if (autoSave.exists) {
                    _emulator.loadState(autoSave.file)
                }
            }

            _gameRepo.markAsPlayed(game.id, uri)
            _loadedGameId.value = game.id
            lastEvent.value = GameEvent.Opened

            true
        } catch (ex: IllegalArgumentException) {
            Toast.makeText(_application, ex.localizedMessage, Toast.LENGTH_LONG).show()
            false
        } catch (ex: Exception) {
            if (ACRA.isInitialised) {
                ACRA.errorReporter.handleException(ex)
            } else {
                Toast.makeText(_application, ex.localizedMessage, Toast.LENGTH_LONG).show()
                Log.e("MainViewModel", ex.localizedMessage, ex)
            }
            false
        }
    }

    fun closeGame() {
        _emulator.unloadGamePak()
        _loadedGameId.value = null
        lastEvent.value = GameEvent.Closed
    }

    fun resetGame() {
        _emulator.reset()
        lastEvent.value = GameEvent.Opened
    }

    fun openGame() {
        lastEvent.value = GameEvent.Opened
    }

    fun configureAutoSave(enabled: Boolean) {
        _loadedGameId.value?.also {
            _gameRepo.setAutoSave(it, enabled)
            if (enabled) {
                val autoSave = _gameRepo.getAutoSave(it)
                _emulator.setAutoSaveFile(autoSave.file)
            } else {
                _emulator.setAutoSaveFile(null)
            }
        }
    }

    fun saveState() {
        currentStateSlot.value?.also {
            _emulator.saveState(it.file)
        }
    }

    fun loadState() {
        currentStateSlot.value?.also {
            _emulator.loadState(it.file)
        }
    }

    fun selectStateSlot(slot: Int) {
        _loadedGameId.value?.also {
            _gameRepo.selectStateSlot(it, slot)
        }
    }

    val recentGames by _gameRepo::recentGames

    private fun <T> forCurrentGame(getter: (String) -> Flow<T>): Flow<T?> {
        return _loadedGameId
            .flatMapLatest { id -> id?.let(getter) ?: emptyFlow() }
    }
}