package com.simongellis.vvb.data

import android.content.Context
import android.os.FileObserver
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.*
import java.io.File

class FileDao(private val scope: CoroutineScope, context: Context) {
    private val _filesDir = context.filesDir
    private val _observers = HashMap<String, DirectoryObserver>()

    fun get(filename: String): File {
        val file = File(_filesDir, filename)
        file.parentFile?.mkdirs()
        return file
    }

    fun watch(filename: String): Flow<File> {
        val file = File(_filesDir, filename)
        val name = file.name
        return getDirectory(file).updateFlow
            .filter { it == name }
            .map { file }
            .onStart { emit(file) }
    }

    private fun getDirectory(file: File) = _observers.getOrPut(file.parent!!) {
        DirectoryObserver(scope, file.parentFile!!)
    }

    class DirectoryObserver(scope: CoroutineScope, directory: File) {
        val updateFlow = getUpdateFlow(directory)
            .shareIn(scope, SharingStarted.WhileSubscribed())

        init {
            directory.mkdirs()
        }

        private fun getUpdateFlow(directory: File) = callbackFlow {
            @Suppress("DEPRECATION")
            val observer = object : FileObserver(directory.path, CLOSE_WRITE) {
                override fun onEvent(event: Int, path: String?) {
                    path?.also(::trySend)
                }
            }
            observer.startWatching()
            awaitClose { observer.stopWatching() }
        }
    }
}