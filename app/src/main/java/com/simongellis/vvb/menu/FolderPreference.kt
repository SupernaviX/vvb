package com.simongellis.vvb.menu

import android.content.Context
import android.net.Uri
import android.provider.OpenableColumns
import android.util.AttributeSet
import androidx.fragment.app.Fragment
import androidx.preference.Preference

class FolderPreference(private val context: Context, attrs: AttributeSet): Preference(context, attrs) {
    private var folderPicker: FolderPicker? = null

    var value: Uri? = null
        private set

    fun initialize(fragment: Fragment) {
        folderPicker = FolderPicker(fragment, ::setValue)
    }

    override fun onClick() {
        folderPicker?.open()
    }

    override fun onSetInitialValue(defaultValue: Any?) {
        getPersistedString(defaultValue as? String)?.also {
            value = Uri.parse(it)
        }
    }

    override fun getSummary(): CharSequence {
        return value?.let { uri ->
            val cursor = try {
                context.contentResolver.query(
                    uri,
                    arrayOf(OpenableColumns.DISPLAY_NAME),
                    null,
                    null,
                    null,
                )
            } catch (ex: Exception) {
                return uri.path ?: ""
            }
            cursor?.use {
                if (it.moveToFirst()) {
                    return it.getString(it.getColumnIndexOrThrow(OpenableColumns.DISPLAY_NAME))
                }
            }
            uri.path
        } ?: ""
    }

    fun setValue(newValue: Uri?) {
        if (value != newValue) {
            value = newValue
            persistString(value?.toString())
            notifyChanged()
        }
    }
}