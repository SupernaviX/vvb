package com.simongellis.vvb.menu

import android.content.SharedPreferences
import android.net.Uri
import com.fredporciuncula.flow.preferences.FlowSharedPreferences
import com.simongellis.vvb.game.GamePakLoader
import com.simongellis.vvb.utils.asStateFlow
import kotlinx.coroutines.CoroutineScope

class RecentGamesDao(scope: CoroutineScope, preferences: SharedPreferences) {
    private val _preferences = FlowSharedPreferences(preferences)
    private val _rawRecentGames = _preferences.getStringSet("recent_games", setOf())

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
        _rawRecentGames.asStateFlow(scope) { parseRecentGames(it) }
    }

    private fun parseRecentGames(raw: Set<String>): List<RecentGame> {
        return raw
            .map { RecentGame.fromString(it) }
            .sortedByDescending { it.lastPlayed }
    }

    fun addRecentGame(uri: Uri) {
        val game = RecentGame(System.currentTimeMillis(), uri)
        val otherGames = parseRecentGames(_rawRecentGames.get()).filter { it.uri != uri }
        val recentGames = listOf(game).plus(otherGames).take(10)
        _rawRecentGames.set(recentGames.map { it.toString() }.toSet())
    }
}