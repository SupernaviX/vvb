package com.simongellis.vvb

import android.app.Application
import android.net.Uri
import android.util.Log
import android.widget.Toast
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import com.simongellis.vvb.data.BundledGameRepository
import com.simongellis.vvb.data.GameRepository
import com.simongellis.vvb.emulator.Emulator
import com.simongellis.vvb.emulator.GamePak
import com.simongellis.vvb.emulator.StateSlot
import com.simongellis.vvb.game.GamePakLoader
import kotlinx.coroutines.flow.*
import org.acra.ACRA
import java.lang.Exception

class MainViewModel(application: Application): AndroidViewModel(application) {
    private val _gameRepo = GameRepository(viewModelScope, application)
    private val _bundledGameRepo = BundledGameRepository(application)
    private val _application = getApplication<VvbApplication>()
    private val _emulator = Emulator.instance
    private val _gamePakLoader = GamePakLoader(application)

    private val _loadedGamePak = MutableStateFlow<GamePak?>(null)
    val loadedGame = forCurrentGame { _gameRepo.watchGame(it.hash) }
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
            val gamePak = _gamePakLoader.load(uri)
            gamePak.initFilesystem()
            val data = _gameRepo.getGameData(gamePak.hash, uri)
            val autoSave = gamePak.autoStateSlot

            _emulator.loadGamePak(gamePak, data.autoSaveEnabled)
            if (data.autoSaveEnabled && autoSave.exists) {
                loadState(autoSave)
            }

            _gameRepo.markAsPlayed(data.id, uri)
            _loadedGamePak.value = gamePak
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
        _loadedGamePak.value = null
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
        _loadedGamePak.value?.also {
            _gameRepo.setAutoSaveEnabled(it.hash, enabled)
            _emulator.setAutoSaveEnabled(enabled)
        }
    }

    fun saveState() {
        currentStateSlot.value?.also(this::saveState)
    }

    private fun saveState(slot: StateSlot) {
        _emulator.saveState(slot.file)
        Toast.makeText(_application, R.string.toast_state_saved, Toast.LENGTH_SHORT).show()
    }

    fun loadState() {
        currentStateSlot.value?.also(this::loadState)
    }

    private fun loadState(slot: StateSlot) {
        _emulator.loadState(slot.file)
        Toast.makeText(_application, R.string.toast_state_loaded, Toast.LENGTH_SHORT).show()
    }

    fun selectStateSlot(slot: Int) {
        _loadedGamePak.value?.also {
            _gameRepo.selectStateSlot(it.hash, slot)
        }
    }

    val recentGames by _gameRepo::recentGames
    val hasRecentGames
        get() = _gameRepo.hasRecentGames()
    val bundledGames by _bundledGameRepo::bundledGames

    private fun <T> forCurrentGame(getter: (GamePak) -> Flow<T>): Flow<T?> {
        return _loadedGamePak
            .flatMapLatest { pak -> pak?.let(getter) ?: emptyFlow() }
    }
}