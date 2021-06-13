package com.simongellis.vvb.menu

import android.content.SharedPreferences
import android.net.Uri
import androidx.core.content.edit

class RecentGamesDao(private val preferences: SharedPreferences) {
    data class RecentGame(val lastPlayed: Long, val uri: Uri) {
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

    fun getRecentGames(): List<RecentGame> {
        return preferences.getStringSet("recent_games", setOf())!!
            .map { RecentGame.fromString(it) }
            .sortedByDescending { it.lastPlayed }
    }

    fun addRecentGame(uri: Uri) {
        val game = RecentGame(System.currentTimeMillis(), uri)
        val otherGames = getRecentGames().filter { it.uri != uri }
        val recentGames = listOf(game).plus(otherGames).take(3)
        preferences.edit {
            putStringSet("recent_games", recentGames.map { it.toString() }.toSet())
        }
    }
}