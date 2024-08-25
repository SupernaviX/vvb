package com.simongellis.vvb.menu

import android.app.Dialog
import android.content.Intent
import android.icu.text.ListFormatter
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.text.SpannableString
import android.text.method.LinkMovementMethod
import android.text.util.Linkify
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView
import androidx.annotation.StringRes
import androidx.appcompat.app.AlertDialog
import androidx.core.content.edit
import androidx.core.view.isVisible
import androidx.fragment.app.Fragment
import androidx.fragment.app.viewModels
import androidx.preference.PreferenceManager
import androidx.recyclerview.widget.*
import com.simongellis.vvb.MainViewModel
import com.simongellis.vvb.R
import com.simongellis.vvb.data.BundledGame
import com.simongellis.vvb.data.Game
import com.simongellis.vvb.databinding.MenuItemBinding
import com.simongellis.vvb.game.GameActivity
import com.simongellis.vvb.utils.observeNow
import kotlin.properties.Delegates

class LoadGameMenuFragment : Fragment() {
    private val viewModel: MainViewModel by viewModels({ requireActivity() })

    private lateinit var _loadGame: LoadFromFileAdapter
    private val _recentGames = RecentGamesListAdapter(::loadGame)
    private val _bundledGames = BundledGamesListAdapter(::loadGame)

    private var _dialog: Dialog? = null
        set(value) {
            field = value
            value?.setOnDismissListener { field = null }
        }

    private companion object {
        const val EXPAND_RECENT_GAMES = "EXPAND_RECENT_GAMES"
        const val EXPAND_BUNDLED_GAMES = "EXPAND_BUNDLED_GAMES"
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val prefs = PreferenceManager.getDefaultSharedPreferences(requireContext())

        val fileLoader = FilePicker(this, FilePicker.Mode.Read, ::loadGame)
        _loadGame = LoadFromFileAdapter {
            if (prefs.contains("seen_load_game_explanation")) {
                fileLoader.open()
            } else {
                showLoadGameExplanation {
                    prefs.edit { putBoolean("seen_load_game_explanation", true) }
                    fileLoader.open()
                }
            }
        }

        observeNow(viewModel.recentGames) {
            _recentGames.items = it
        }

        _bundledGames.items = viewModel.bundledGames

        _recentGames.expanded = savedInstanceState?.getBoolean(EXPAND_RECENT_GAMES) ?: viewModel.hasRecentGames
        _bundledGames.expanded = savedInstanceState?.getBoolean(EXPAND_BUNDLED_GAMES) ?: true
    }

    override fun onSaveInstanceState(outState: Bundle) {
        super.onSaveInstanceState(outState)
        outState.putBoolean(EXPAND_RECENT_GAMES, _recentGames.expanded)
        outState.putBoolean(EXPAND_BUNDLED_GAMES, _bundledGames.expanded)
    }

    override fun onResume() {
        super.onResume()
        requireActivity().setTitle(R.string.main_menu_load_game)
    }

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?,
    ): View {
        return RecyclerView(requireContext()).apply {
            layoutManager = LinearLayoutManager(context)
            adapter = ConcatAdapter(
                _loadGame,
                *_recentGames.adapters,
                *_bundledGames.adapters)
        }
    }

    private fun loadGame(uri: Uri?) {
        uri?.also {
            if (viewModel.loadGame(it)) {
                playGame()
            }
        }
    }

    private fun showLoadGameExplanation(onConfirm: () -> Unit) {
        val context = requireContext()
        val message = SpannableString(context.getString(R.string.load_game_from_file_explanation))
        Linkify.addLinks(message, Linkify.WEB_URLS)

        _dialog = AlertDialog.Builder(requireContext())
            .setTitle(R.string.load_game_from_file)
            .setMessage(message)
            .setPositiveButton(R.string.load_game_from_file_explanation_button) { _, _ -> onConfirm() }
            .create()

        _dialog?.apply {
            show()
            findViewById<TextView>(android.R.id.message)?.movementMethod = LinkMovementMethod.getInstance()
        }
    }

    private fun playGame() {
        val intent = Intent(requireActivity(), GameActivity::class.java)
        startActivity(intent)
    }

    class MenuItemViewHolder(val binding: MenuItemBinding) : RecyclerView.ViewHolder(binding.root)

    abstract class MenuItemAdapter : RecyclerView.Adapter<MenuItemViewHolder>() {
        override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): MenuItemViewHolder {
            val binding = MenuItemBinding.inflate(LayoutInflater.from(parent.context))
            return MenuItemViewHolder(binding)
        }

        override fun getItemCount(): Int {
            return 1
        }
    }

    class LoadFromFileAdapter(val onClick: () -> Unit) : MenuItemAdapter() {
        override fun onBindViewHolder(holder: MenuItemViewHolder, position: Int) {
            holder.binding.title.setText(R.string.load_game_from_file)
            holder.binding.root.setOnClickListener { onClick() }
        }
    }

    class RecentGamesListAdapter(val loadGame: (uri: Uri) -> Unit) :
        SimpleListAdapter<Game>(R.string.load_game_recent_games,
            R.string.load_game_no_recent_games) {
        override fun onBindMenuItem(binding: MenuItemBinding, item: Game) {
            binding.title.text = item.name
            binding.root.setOnClickListener { loadGame(item.uri) }
        }

        override fun areItemsTheSame(oldItem: Game, newItem: Game): Boolean {
            return oldItem.id == newItem.id
        }

        override fun areContentsTheSame(oldItem: Game, newItem: Game): Boolean {
            return oldItem == newItem
        }
    }

    class BundledGamesListAdapter(val loadGame: (uri: Uri) -> Unit) :
        SimpleListAdapter<BundledGame>(R.string.load_game_bundled_games,
            R.string.load_game_no_bundled_games) {
        override fun onBindMenuItem(binding: MenuItemBinding, item: BundledGame) {
            binding.title.text = item.name
            binding.summary.isVisible = true

            binding.summary.text = binding.root.context.getString(R.string.load_game_created_by, authorsToString(item.authors))
            binding.root.setOnClickListener { loadGame(item.uri) }
        }

        override fun areItemsTheSame(oldItem: BundledGame, newItem: BundledGame): Boolean {
            return oldItem.id == newItem.id
        }

        override fun areContentsTheSame(oldItem: BundledGame, newItem: BundledGame): Boolean {
            return oldItem == newItem
        }

        private fun authorsToString(authors: List<String>): String {
            return if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                ListFormatter.getInstance().format(authors)
            } else {
                authors.joinToString(", ")
            }
        }

    }

    abstract class SimpleListAdapter<T : Any>(
        @StringRes val titleText: Int,
        @StringRes val noEntriesText: Int,
    ) {
        var expanded: Boolean by Delegates.observable(false) { _, oldValue, newValue ->
            if (!oldValue && newValue) {
                _headerAdapter.notifyItemChanged(0)
                showItems()
            }
            if (oldValue && !newValue) {
                _headerAdapter.notifyItemChanged(0)
                hideItems()
            }
        }
        var items: List<T> by Delegates.observable(listOf()) { _, oldValue, newValue ->
            if (expanded) {
                if (oldValue.isEmpty() && newValue.isNotEmpty()) {
                    _noEntriesAdapter.notifyItemRemoved(0)
                }
                if (oldValue.isNotEmpty() && newValue.isEmpty()) {
                    _noEntriesAdapter.notifyItemInserted(0)
                }
                _entriesAdapter.submitList(newValue.toMutableList())
            }
        }

        abstract fun onBindMenuItem(binding: MenuItemBinding, item: T)
        abstract fun areItemsTheSame(oldItem: T, newItem: T): Boolean
        abstract fun areContentsTheSame(oldItem: T, newItem: T): Boolean

        private fun showItems() {
            if (items.isEmpty()) {
                _noEntriesAdapter.notifyItemInserted(0)
            } else {
                _entriesAdapter.submitList(items.toMutableList())
            }
        }

        private fun hideItems() {
            if (items.isEmpty()) {
                _noEntriesAdapter.notifyItemRemoved(0)
            } else {
                _entriesAdapter.submitList(null)
            }
        }

        private val _differ = object : DiffUtil.ItemCallback<T>() {
            override fun areItemsTheSame(oldItem: T, newItem: T): Boolean {
                return this@SimpleListAdapter.areItemsTheSame(oldItem, newItem)
            }

            override fun areContentsTheSame(oldItem: T, newItem: T): Boolean {
                return this@SimpleListAdapter.areContentsTheSame(oldItem, newItem)
            }
        }

        private val _headerAdapter = object : MenuItemAdapter() {
            override fun onBindViewHolder(holder: MenuItemViewHolder, position: Int) {
                holder.binding.title.setText(titleText)
                holder.binding.icon.setImageResource(R.drawable.ic_arrow_down_24)
                holder.binding.icon.rotation = if (expanded) {
                    180f
                } else {
                    0f
                }
                holder.binding.root.setOnClickListener {
                    expanded = !expanded
                }
            }
        }

        private val _noEntriesAdapter = object : MenuItemAdapter() {
            override fun onBindViewHolder(holder: MenuItemViewHolder, position: Int) {
                holder.binding.title.setText(noEntriesText)
            }

            override fun getItemCount(): Int {
                return if (expanded && items.isEmpty()) {
                    1
                } else {
                    0
                }
            }
        }

        private val _entriesAdapter = object : ListAdapter<T, MenuItemViewHolder>(_differ) {
            override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): MenuItemViewHolder {
                val holder = MenuItemBinding.inflate(LayoutInflater.from(parent.context))
                return MenuItemViewHolder(holder)
            }

            override fun onBindViewHolder(holder: MenuItemViewHolder, position: Int) {
                onBindMenuItem(holder.binding, items[position])
            }
        }

        val adapters = arrayOf(_headerAdapter, _noEntriesAdapter, _entriesAdapter)
    }
}