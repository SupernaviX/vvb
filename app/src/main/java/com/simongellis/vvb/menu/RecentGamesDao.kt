package com.simongellis.vvb.menu

import android.content.SharedPreferences
import android.net.Uri
import androidx.core.content.edit
import com.simongellis.vvb.game.GamePakLoader
import com.simongellis.vvb.utils.PreferenceLiveDataSource

class RecentGamesDao(private val preferences: SharedPreferences) {
    private val _dataSource = PreferenceLiveDataSource(preferences)

    data class RecentGame(val lastPlayed: Long, val uri: Uri) {
        val name = GamePakLoader.getName(uri)

        override fun toString(): String {
            return "$lastPlayed::$uri"
        }
        companion object {
            fun fromString(value: String): RecentGame {
                val (lastPlayed, uri) = value.split("::", limit = 2)
                return RecentGame(lastPlayed.toLong(), Uri.parse(uri))
            }
        }
    }

    val recentGames by lazy {
        _dataSource.get("recent_games", this::getRecentGames)
    }

    private fun getRecentGames(): List<RecentGame> {
        return preferences.getStringSet("recent_games", setOf())!!
            .map { RecentGame.fromString(it) }
            .sortedByDescending { it.lastPlayed }
    }

    fun addRecentGame(uri: Uri) {
        val game = RecentGame(System.currentTimeMillis(), uri)
        val otherGames = getRecentGames().filter { it.uri != uri }
        val recentGames = listOf(game).plus(otherGames).take(10)
        preferences.edit {
            putStringSet("recent_games", recentGames.map { it.toString() }.toSet())
        }
    }
}