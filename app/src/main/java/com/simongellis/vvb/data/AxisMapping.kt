package com.simongellis.vvb.data

import android.view.MotionEvent
import com.simongellis.vvb.emulator.Input
import kotlinx.serialization.Serializable

@Serializable
data class AxisMapping(
    override val device: String,
    override val input: Input,
    val axis: Int,
    val isNegative: Boolean
): Mapping {
    override fun toString(): String {
        val sign = if (isNegative) { '-' } else { '+' }
        return "${MotionEvent.axisToString(axis).removePrefix("AXIS_")} $sign"
    }
}