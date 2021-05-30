package com.simongellis.vvb.emulator

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
        val state = _activeInputs or input.bitMask
        update(state)
    }

    fun release(input: Input) {
        val state = _activeInputs and input.bitMask.inv()
        update(state)
    }

    fun update(pressed: List<Input>, released: List<Input>) {
        var state = _activeInputs
        for (input in pressed) {
            state = state or input.bitMask
        }
        for (input in released) {
            state = state and input.bitMask.inv()
        }
        update(state)
    }

    private fun update(state: Int) {
        if (state != _activeInputs) {
            _activeInputs = state
            nativeUpdate(_activeInputs)
        }
    }

    private external fun nativeConstructor(emulator: Emulator)
    private external fun nativeDestructor()
    private external fun nativeUpdate(state: Int)
}