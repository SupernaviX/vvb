package com.simongellis.vvb

class Controller(emulator: Emulator) {
    private var _pointer = 0L
    private var _activeInputs = Input.SIGNATURE.bitMask

    init {
        nativeConstructor(emulator)
        nativeUpdate(_activeInputs)
    }

    fun finalize() {
        destroy()
    }

    fun destroy() {
        if (_pointer != 0L) {
            nativeDestructor()
        }
    }

    fun press(input: Input) {
        _activeInputs = _activeInputs or input.bitMask
        nativeUpdate(_activeInputs)
    }

    fun release(input: Input) {
        _activeInputs = _activeInputs and input.bitMask.inv()
        nativeUpdate(_activeInputs)
    }

    private external fun nativeConstructor(emulator: Emulator)
    private external fun nativeDestructor()
    private external fun nativeUpdate(state: Int)
}