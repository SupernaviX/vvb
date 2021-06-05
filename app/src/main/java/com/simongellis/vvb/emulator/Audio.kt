package com.simongellis.vvb.emulator

class Audio(emulator: Emulator, settings: Settings) {
    private var _pointer = 0L

    init {
        nativeConstructor(emulator, settings)
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

    class Settings(val volume: Float, val bufferSize: Int)

    private external fun nativeConstructor(emulator: Emulator, settings: Settings)
    private external fun nativeDestructor()
    private external fun nativePlay()
    private external fun nativePause()
}