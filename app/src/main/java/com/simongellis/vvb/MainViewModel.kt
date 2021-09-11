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
    private val _gameRepo = GameRepository(application)
    private val _application = getApplication<VvbApplication>()
    private val _emulator = Emulator.instance
    private val _gamePakLoader = GamePakLoader(application)

    private val _loadedGameId = MutableStateFlow<String?>(null)
    val loadedGame = _loadedGameId.flatMapLatest { id ->
        id?.let { _gameRepo.watchGame(it) } ?: emptyFlow()
    }.stateIn(viewModelScope, SharingStarted.WhileSubscribed(), null)

    var wasGameJustOpened = false
    fun loadGame(uri: Uri): Boolean {
        return try {
            val game = _gameRepo.getGame(uri)
            val gamePak = _gamePakLoader.tryLoad(game.id, uri)
            _emulator.loadGamePak(gamePak)
            _gameRepo.markAsPlayed(game.id, uri)
            _loadedGameId.value = game.id
            wasGameJustOpened = true
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
    }

    fun resetGame() {
        _emulator.reset()
        wasGameJustOpened = true
    }

    fun openGame() {
        wasGameJustOpened = true
    }

    fun saveState() {
        loadedGame.value?.also {
            _emulator.saveState(it.currentState.file)
        }
    }

    fun loadState() {
        loadedGame.value?.also {
            _emulator.loadState(it.currentState.file)
        }
    }

    val recentGames by _gameRepo::recentGames
}