package com.simongellis.vvb.menu

import android.content.Context
import android.util.AttributeSet
import androidx.preference.ListPreference

open class DetailedListPreference(context: Context, attrs: AttributeSet): ListPreference(context, attrs) {
    var detailedEntries: List<Entry> = listOf()

    override fun getEntries(): Array<CharSequence> {
        return detailedEntries.map { it.summary }.toTypedArray()
    }

    override fun getEntryValues(): Array<CharSequence> {
        return detailedEntries.map { it.value }.toTypedArray()
    }

    override fun findIndexOfValue(value: String): Int {
        return detailedEntries.indexOfFirst { it.value == value }
    }

    override fun getSummary(): CharSequence {
        val selectedIndex = findIndexOfValue(value)
        return if (selectedIndex >= 0 && selectedIndex < detailedEntries.size) {
            detailedEntries[selectedIndex].summary
        } else {
            super.getSummary()
        }
    }

    data class Entry(
        val value: String,
        val summary: String,
        val description: String,
    )
}