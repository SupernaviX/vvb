package com.simongellis.vvb

import android.app.Application
import android.content.Intent
import android.net.Uri
import androidx.lifecycle.AndroidViewModel
import androidx.preference.PreferenceManager
import com.simongellis.vvb.emulator.Emulator
import com.simongellis.vvb.menu.RecentGamesDao

class MainViewModel(application: Application): AndroidViewModel(application) {
    private val _preferences = PreferenceManager.getDefaultSharedPreferences(application)
    private val _recentGamesDao = RecentGamesDao(_preferences)
    private val _application = getApplication<VvbApplication>()

    val isGameLoaded get() = Emulator.instance.isGameLoaded()
    fun loadGame(uri: Uri) {
        _application.contentResolver.takePersistableUriPermission(uri, Intent.FLAG_GRANT_READ_URI_PERMISSION)

        Emulator.instance.loadGamePak(_application, uri)
        _recentGamesDao.addRecentGame(uri)
    }

    val recentGames by _recentGamesDao::recentGames
}