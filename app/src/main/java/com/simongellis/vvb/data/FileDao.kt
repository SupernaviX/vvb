package com.simongellis.vvb.data

import android.content.Context
import android.os.FileObserver
import kotlinx.coroutines.flow.*
import java.io.File

class FileDao(context: Context) {
    private val _filesDir = context.filesDir
    private val _observers = HashMap<String, FlowFileObserver>()

    fun get(filename: String): File {
        val file = File(_filesDir, filename)
        file.parentFile?.mkdirs()
        return file
    }

    fun watch(filename: String) = _observers.getOrPut(filename) {
        val observer = FlowFileObserver(get(filename))
        observer.startWatching()
        observer
    }.flow

    @Suppress("DEPRECATION")
    class FlowFileObserver(file: File): FileObserver(file.parent, CLOSE_WRITE) {
        private val _path = file.name
        private val _eventFlow = MutableSharedFlow<Unit>(replay = 1)
        val flow = _eventFlow.map { file }

        init {
            _eventFlow.tryEmit(Unit)
        }

        override fun onEvent(event: Int, path: String?) {
            if (_path == path) {
                _eventFlow.tryEmit(Unit)
            }
        }
    }
}