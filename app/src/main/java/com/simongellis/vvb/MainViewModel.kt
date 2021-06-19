package com.simongellis.vvb

import android.app.Application
import android.content.Intent
import android.net.Uri
import android.util.Log
import android.widget.Toast
import androidx.lifecycle.AndroidViewModel
import androidx.preference.PreferenceManager
import com.simongellis.vvb.emulator.Emulator
import com.simongellis.vvb.menu.RecentGamesDao
import org.acra.ACRA
import java.lang.Exception

class MainViewModel(application: Application): AndroidViewModel(application) {
    private val _preferences = PreferenceManager.getDefaultSharedPreferences(application)
    private val _recentGamesDao = RecentGamesDao(_preferences)
    private val _application = getApplication<VvbApplication>()

    val isGameLoaded get() = Emulator.instance.isGameLoaded()
    fun loadGame(uri: Uri): Boolean {
        _application.contentResolver.takePersistableUriPermission(uri, Intent.FLAG_GRANT_READ_URI_PERMISSION)

        return try {
            Emulator.instance.tryLoadGamePak(_application, uri)
            _recentGamesDao.addRecentGame(uri)
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

    val recentGames by _recentGamesDao::recentGames
}