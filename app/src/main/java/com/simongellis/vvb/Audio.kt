package com.simongellis.vvb

class Audio(emulator: Emulator) {
    private var _pointer = 0L

    init {
        nativeConstructor(emulator)
    }

    fun finalize() {
        destroy()
    }

    fun destroy() {
        if (_pointer != 0L) {
            nativeDestructor()
        }
    }

    fun play() {
        nativePlay()
    }

    fun pause() {
        nativePause()
    }

    private external fun nativeConstructor(emulator: Emulator)
    private external fun nativeDestructor()
    private external fun nativePlay()
    private external fun nativePause()
}