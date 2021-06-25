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

    fun start() {
        nativeStart()
    }

    fun stop() {
        nativeStop()
    }

    class Settings(val volume: Float, val bufferSize: Int)

    private external fun nativeConstructor(emulator: Emulator, settings: Settings)
    private external fun nativeDestructor()
    private external fun nativeStart()
    private external fun nativeStop()
}