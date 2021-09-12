package com.simongellis.vvb.menu

import android.content.Context
import android.util.AttributeSet
import androidx.core.content.res.use
import androidx.preference.ListPreference
import com.simongellis.vvb.R

class DetailedListPreference(context: Context, attrs: AttributeSet): ListPreference(context, attrs) {
    var detailedEntries: List<Entry> = listOf()
        set(value) {
            field = value
            notifyChanged()
        }
    private val summaryFormat: String
    init {
        var format = "%1\$s"
        context.obtainStyledAttributes(attrs, R.styleable.DetailedListPreference).use { a ->
            a.getString(R.styleable.DetailedListPreference_summaryFormat)
                ?.let { format = it }
        }
        summaryFormat = format
    }


    override fun getEntries(): Array<CharSequence> {
        return detailedEntries.map { it.name }.toTypedArray()
    }

    override fun getEntryValues(): Array<CharSequence> {
        return detailedEntries.map { it.value }.toTypedArray()
    }

    override fun findIndexOfValue(value: String): Int {
        return detailedEntries.indexOfFirst { it.value == value }
    }

    override fun getSummary(): CharSequence {
        val selectedIndex = findIndexOfValue(value)
        if (selectedIndex < 0 || selectedIndex >= detailedEntries.size) {
            return ""
        }
        val entry = detailedEntries[selectedIndex]
        return String.format(summaryFormat, entry.name, entry.description)
    }

    data class Entry(
        val value: String,
        val name: String,
        val description: String,
    )
}