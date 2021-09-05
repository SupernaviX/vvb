package com.simongellis.vvb.data

import android.view.KeyEvent
import com.simongellis.vvb.emulator.Input
import kotlinx.serialization.Serializable

@Serializable
data class KeyMapping(
    override val device: String,
    override val input: Input,
    val keyCode: Int
): Mapping {
    override fun toString(): String {
        return KeyEvent.keyCodeToString(keyCode).removePrefix("KEYCODE_")
    }
}
