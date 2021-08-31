package com.simongellis.vvb

import android.app.Application
import android.net.Uri
import android.util.Log
import android.widget.Toast
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.asLiveData
import androidx.preference.PreferenceManager
import com.simongellis.vvb.emulator.Emulator
import com.simongellis.vvb.game.GamePakLoader
import com.simongellis.vvb.menu.RecentGamesDao
import org.acra.ACRA
import java.lang.Exception

class MainViewModel(application: Application): AndroidViewModel(application) {
    private val _preferences = PreferenceManager.getDefaultSharedPreferences(application)
    private val _recentGamesDao = RecentGamesDao(_preferences)
    private val _application = getApplication<VvbApplication>()
    private val _emulator = Emulator.instance
    private val _gamePakLoader = GamePakLoader(application)

    val isGameLoaded get() = _emulator.isGameLoaded()
    var wasGameJustLoaded = false
    fun loadGame(uri: Uri): Boolean {
        return try {
            val gamePak = _gamePakLoader.tryLoad(uri)
            _emulator.loadGamePak(gamePak)
            _recentGamesDao.addRecentGame(uri)
            wasGameJustLoaded = true
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

    val recentGames by lazy {
        _recentGamesDao.recentGames.asLiveData()
    }
}