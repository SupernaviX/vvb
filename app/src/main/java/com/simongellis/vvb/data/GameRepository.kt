package com.simongellis.vvb.data

import android.content.Context
import android.net.Uri
import kotlinx.coroutines.flow.map
import java.util.*

class GameRepository(context: Context) {
    private val dao = PreferencesDao.forClass(Game.serializer(), context)

    val recentGames by lazy {
        dao.watchAll()
            .map { games -> games.sortedByDescending { it.lastPlayed }.take(10) }
    }

    fun addGame(uri: Uri) {
        dao.put(Game(uri, Date()))
    }
}