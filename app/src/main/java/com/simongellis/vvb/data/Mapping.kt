package com.simongellis.vvb.data

import com.simongellis.vvb.emulator.Input

interface Mapping {
    val device: String
    val input: Input
}
