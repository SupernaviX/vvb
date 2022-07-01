package com.simongellis.vvb

import android.content.ContentProvider
import android.content.ContentValues
import android.content.res.AssetFileDescriptor
import android.database.Cursor
import android.database.MatrixCursor
import android.net.Uri
import android.os.Bundle
import android.os.CancellationSignal
import java.io.IOException

class AssetsProvider : ContentProvider() {
    override fun onCreate(): Boolean {
        return true
    }

    override fun openAssetFile(uri: Uri, mode: String): AssetFileDescriptor? {
        return getFilePath(uri)?.let { context?.assets?.openFd(it) }
    }

    override fun openAssetFile(
        uri: Uri,
        mode: String,
        signal: CancellationSignal?,
    ): AssetFileDescriptor? {
        return openAssetFile(uri, mode)
    }

    override fun openTypedAssetFile(
        uri: Uri,
        mimeTypeFilter: String,
        opts: Bundle?
    ): AssetFileDescriptor? {
        return openAssetFile(uri, "r")
    }

    override fun openTypedAssetFile(
        uri: Uri,
        mimeTypeFilter: String,
        opts: Bundle?,
        signal: CancellationSignal?
    ): AssetFileDescriptor? {
        return openTypedAssetFile(uri, mimeTypeFilter, opts)
    }

    override fun query(
        uri: Uri,
        projection: Array<out String>?,
        selection: String?,
        selectionArgs: Array<out String>?,
        sortOrder: String?,
    ): Cursor? {
        val path = getFilePath(uri) ?: return null
        val exists = try {
            context?.assets?.openFd(path) != null
        } catch (e: IOException) {
            false
        }
        val cursor = MatrixCursor(projection)
        if (exists) {
            val displayName = path.substringAfterLast("/")
            cursor.addRow(arrayOf(displayName))
        }
        return cursor
    }

    private fun getFilePath(uri: Uri): String? {
        return uri.path?.substring(1)
    }

    override fun getType(uri: Uri): String {
        return "application/octet-stream"
    }

    override fun getStreamTypes(uri: Uri, mimeTypeFilter: String): Array<String> {
        return arrayOf("application/octet-stream")
    }

    override fun insert(p0: Uri, p1: ContentValues?): Uri? {
        return null
    }

    override fun delete(p0: Uri, p1: String?, p2: Array<out String>?): Int {
        return 0
    }

    override fun update(p0: Uri, p1: ContentValues?, p2: String?, p3: Array<out String>?): Int {
        return 0
    }
}