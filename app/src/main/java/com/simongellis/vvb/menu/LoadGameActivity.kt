package com.simongellis.vvb.menu

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.activity.viewModels
import androidx.annotation.StringRes
import androidx.appcompat.app.AppCompatActivity
import androidx.core.view.isVisible
import androidx.recyclerview.widget.*
import com.simongellis.vvb.MainViewModel
import com.simongellis.vvb.R
import com.simongellis.vvb.databinding.TextSummaryBinding
import com.simongellis.vvb.game.GameActivity
import java.lang.IllegalArgumentException
import kotlin.properties.Delegates

class LoadGameActivity: AppCompatActivity() {
    private val viewModel: MainViewModel by viewModels()

    private val _recentGames = GameListAdapter(R.string.load_game_no_recent_games).apply {
        games = mutableListOf("foo", "bar", "baz", "quux", "xyzzy", "make", "up", "some", "more", "words", "please")
    }
    private val _recentGamesHeader = GameListHeaderAdapter(R.string.load_game_recent_games, _recentGames)
    private val _bundledGames = GameListAdapter(R.string.load_game_no_bundled_games)
    private val _bundledGamesHeader = GameListHeaderAdapter(R.string.load_game_bundled_games, _bundledGames)

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val context = this

        val fileLoader = GameFilePicker(this, this::loadGame)
        val loadGameAdapter = LoadGameAdapter(fileLoader::open)
        setContentView(RecyclerView(context).apply {
            layoutManager = LinearLayoutManager(context)
            adapter = ConcatAdapter(
                loadGameAdapter,
                _recentGamesHeader,
                _recentGames,
                _bundledGamesHeader,
                _bundledGames)
        })
    }

    private fun loadGame(uri: Uri?) {
        uri?.also {
            if (viewModel.loadGame(it)) {
                playGame()
            }
        }
    }

    private fun playGame() {
        val intent = Intent(this, GameActivity::class.java)
        startActivity(intent)
    }

    class MenuItemViewHolder(val binding: TextSummaryBinding) : RecyclerView.ViewHolder(binding.root)

    abstract class MenuItemAdapter : RecyclerView.Adapter<MenuItemViewHolder>() {
        override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): MenuItemViewHolder {
            val binding = TextSummaryBinding.inflate(LayoutInflater.from(parent.context))
            return MenuItemViewHolder(binding)
        }

        override fun getItemCount(): Int {
            return 1
        }
    }

    class LoadGameAdapter(val onClick: () -> Unit): MenuItemAdapter() {
        override fun onBindViewHolder(holder: MenuItemViewHolder, position: Int) {
            holder.binding.title.setText(R.string.main_menu_load_game)
            holder.binding.root.setOnClickListener { onClick() }
        }
    }

    class GameListHeaderAdapter(@StringRes val titleText: Int, val list: GameListAdapter) : MenuItemAdapter() {
        private var expanded by list::expanded

        override fun onBindViewHolder(holder: MenuItemViewHolder, position: Int) {
            holder.binding.title.setText(titleText)
            holder.binding.icon.setImageResource(R.drawable.ic_arrow_down_24)
            holder.binding.root.setOnClickListener {
                expanded = !expanded
                holder.binding.icon.rotation = if (expanded) { 180f } else { 0f }
            }
        }
    }

    class GameListAdapter(@StringRes val noGamesText: Int): ListAdapter<String, GameListAdapter.ViewHolder>(Differ) {
        var games: MutableList<String> by Delegates.observable(mutableListOf()) { _, _, newValue ->
            if (expanded) {
                submitList(newValue)
            }
        }
        var expanded by Delegates.observable(false) { _, oldValue, newValue ->
            if (!oldValue && newValue) {
                // expanded
                submitList(games)
            }
            if (oldValue && !newValue) {
                // collapsed
                submitList(null)
            }
        }

        companion object {
            private const val GAME = 1
            private const val NO_GAMES = 2

            private val EMPTY_GAME_LIST = listOf("one element")
        }

        override fun submitList(list: MutableList<String>?) {
            if (list?.isEmpty() == true) {
                super.submitList(EMPTY_GAME_LIST)
            } else {
                super.submitList(list)
            }
        }

        object Differ : DiffUtil.ItemCallback<String>() {
            override fun areItemsTheSame(oldItem: String, newItem: String): Boolean {
                return oldItem == newItem
            }

            override fun areContentsTheSame(oldItem: String, newItem: String): Boolean {
                return oldItem == newItem
            }
        }

        sealed class ViewHolder(val view: View): RecyclerView.ViewHolder(view) {
            class GameViewHolder(view: View): GameListAdapter.ViewHolder(view) {
                fun bind(name: String) {
                    val binding = TextSummaryBinding.bind(view)
                    binding.title.text = name
                    binding.summary.isVisible = false
                }
            }
            class NoGamesViewHolder(view: View): GameListAdapter.ViewHolder(view)
        }

        override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): ViewHolder {
            val context = parent.context
            return when (viewType) {
                GAME -> {
                    ViewHolder.GameViewHolder(newView(context).root)
                }
                NO_GAMES -> {
                    val view = newView(context)
                    view.title.setText(noGamesText)
                    ViewHolder.NoGamesViewHolder(view.root)
                }
                else -> throw IllegalArgumentException("Unrecognized view type $viewType")
            }
        }

        private fun newView(context: Context): TextSummaryBinding {
            return TextSummaryBinding.inflate(LayoutInflater.from(context))
        }

        override fun onBindViewHolder(holder: ViewHolder, position: Int) {
            if (holder is ViewHolder.GameViewHolder) {
                holder.bind(games[position])
            }
        }

        override fun getItemViewType(position: Int): Int {
            if (games.isEmpty()) {
                return NO_GAMES
            }
            return GAME
        }
    }
}