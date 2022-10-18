package com.simongellis.vvb.data

import android.os.FileObserver
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.*
import java.io.File

class FileWatcher(private val scope: CoroutineScope) {
    private val _observers = HashMap<String, DirectoryObserver>()

    fun watch(file: File): Flow<File> {
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