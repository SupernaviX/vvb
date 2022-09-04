package com.simongellis.vvb.menu

import android.content.Intent
import android.os.Bundle
import android.text.format.DateFormat
import androidx.fragment.app.viewModels
import androidx.preference.Preference
import androidx.preference.PreferenceFragmentCompat
import androidx.preference.SwitchPreferenceCompat
import com.simongellis.vvb.MainActivity
import com.simongellis.vvb.MainViewModel
import com.simongellis.vvb.R
import com.simongellis.vvb.emulator.StateSlot
import com.simongellis.vvb.game.GameActivity
import com.simongellis.vvb.utils.observeNow
import java.util.*

class GameMenuFragment : PreferenceFragmentCompat() {
    private val viewModel: MainViewModel by viewModels({ requireActivity() })
    private var title: String = ""

    override fun onCreatePreferences(savedInstanceState: Bundle?, rootKey: String?) {
        setPreferencesFromResource(R.xml.preferences_game, rootKey)

        findPreference<Preference>("resume_game")?.setOnPreferenceClickListener {
            viewModel.openGame()
            playGame()
            true
        }
        findPreference<Preference>("reset_game")?.setOnPreferenceClickListener {
            viewModel.resetGame()
            playGame()
            true
        }
        findPreference<SwitchPreferenceCompat>("auto_save")?.setOnPreferenceChangeListener { _, newValue ->
            viewModel.configureAutoSave(newValue as Boolean)
            true
        }
        findPreference<Preference>("save_state")?.setOnPreferenceClickListener {
            viewModel.saveState()
            true
        }
        findPreference<Preference>("load_state")?.setOnPreferenceClickListener {
            viewModel.loadState()
            playGame()
            true
        }
        findPreference<Preference>("state_slot")?.setOnPreferenceChangeListener { _, newValue ->
            val slot = newValue.toString().toInt()
            viewModel.selectStateSlot(slot)
            true
        }
        findPreference<Preference>("close_game")?.setOnPreferenceClickListener {
            viewModel.closeGame()
            closeGameMenu()
            true
        }

        val nowPlaying = requireContext().resources.getString(R.string.main_menu_now_playing)
        title = nowPlaying

        observeNow(viewModel.loadedGame) { game ->
            if (game == null) return@observeNow

            title = "$nowPlaying: ${game.name}"
            findPreference<DetailedListPreference>("state_slot")?.value = game.stateSlot.toString()
            findPreference<SwitchPreferenceCompat>("auto_save")?.isChecked = game.autoSaveEnabled
        }
        observeNow(viewModel.stateSlots) { states ->
            val pref = findPreference<DetailedListPreference>("state_slot")
            val allStates = states ?: listOf()
            pref?.detailedEntries = allStates.map { slot ->
                DetailedListPreference.Entry(
                    slot.name,
                    getSlotName(slot),
                    getSlotDescription(slot)
                )
            }
        }
        observeNow(viewModel.currentStateSlot) {
            val hasSaveState = it?.exists ?: false
            findPreference<Preference>("load_state")?.isEnabled = hasSaveState
        }
    }

    override fun onResume() {
        super.onResume()
        requireActivity().title = title
    }

    private fun playGame() {
        val intent = Intent(activity, GameActivity::class.java)
        startActivity(intent)
    }

    private fun closeGameMenu() {
        val main = activity as MainActivity
        main.closeAllSubMenus()
    }

    private fun getSlotName(slot: StateSlot): String {
        val context = requireContext()
        return context.getString(R.string.game_menu_state_slot_name, slot.name)
    }

    private fun getSlotDescription(slot: StateSlot): String {
        val context = requireContext()
        if (!slot.exists) {
            return context.getString(R.string.game_menu_state_slot_empty)
        }
        val lastSaved = Date(slot.lastModified)
        val dateStr = DateFormat.getMediumDateFormat(context).format(lastSaved)
        val timeStr = DateFormat.getTimeFormat(context).format(lastSaved)
        val dateTimeStr = "$dateStr $timeStr"
        return context.getString(R.string.game_menu_state_slot_saved, dateTimeStr)
    }
}