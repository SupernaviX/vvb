package com.simongellis.vvb.emulator

import java.io.File
import java.nio.ByteBuffer
import java.util.*

class GamePak(val rom: ByteArray, val hash: String, private val gameDataDir: File) {
    private val sram = gameDataDir.resolve(".srm")
    private val saveStatesDir = gameDataDir.resolve("save_states")

    fun initFilesystem() {
        gameDataDir.mkdirs()
        saveStatesDir.mkdir()
    }

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

    val autoStateSlot = getStateSlot("auto")
    fun getStateSlot(index: Int) = getStateSlot(index.toString())
    private fun getStateSlot(name: String): StateSlot {
        val file = saveStatesDir.resolve("$name.sav")
        return StateSlot(file, name)
    }

    companion object {
        const val SRAM_SIZE = 8 * 1024
        private val emptySram = ByteArray(SRAM_SIZE)
    }
}