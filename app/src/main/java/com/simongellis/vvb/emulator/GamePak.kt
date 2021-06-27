package com.simongellis.vvb.emulator

import java.io.File
import java.nio.ByteBuffer
import java.util.*

class GamePak(val name: String, val rom: ByteArray, private val sram: File) {
    fun loadSram(target: ByteBuffer) {
        when {
            sram.exists() -> target.put(sram.readBytes())
            target.hasArray() -> Arrays.fill(target.array(), 0)
            else -> target.put(emptySram)
        }
    }
    fun saveSram(source: ByteBuffer) {
        sram.outputStream().channel.use { it.write(source) }
    }

    companion object {
        const val sramSize = 8 * 1024
        private val emptySram = ByteArray(sramSize)
    }
}