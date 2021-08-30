package com.simongellis.vvb

import android.app.Application
import android.net.Uri
import android.util.Log
import android.widget.Toast
import androidx.lifecycle.AndroidViewModel
import com.simongellis.vvb.data.Game
import com.simongellis.vvb.data.GameRepository
import com.simongellis.vvb.emulator.Emulator
import com.simongellis.vvb.game.GamePakLoader
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import org.acra.ACRA
import java.lang.Exception

class MainViewModel(application: Application): AndroidViewModel(application) {
    private val _gameRepo = GameRepository(application)
    private val _application = getApplication<VvbApplication>()
    private val _emulator = Emulator.instance
    private val _gamePakLoader = GamePakLoader(application)

    private val _loadedGame = MutableStateFlow<Game?>(null)
    val loadedGame = _loadedGame.asStateFlow()

    var wasGameJustOpened = false
    fun loadGame(uri: Uri): Boolean {
        return try {
            val game = _gameRepo.getGame(uri)
            val gamePak = _gamePakLoader.tryLoad(game.id, uri)
            _emulator.loadGamePak(gamePak)
            _gameRepo.markAsPlayed(game.id, uri)
            _loadedGame.value = game
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
        _loadedGame.value = null
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
            _emulator.saveState(it.currentState)
        }
    }

    fun loadState() {
        loadedGame.value?.also {
            _emulator.loadState(it.currentState)
        }
    }

    val recentGames by _gameRepo::recentGames
}