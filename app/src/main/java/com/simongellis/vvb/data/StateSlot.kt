package com.simongellis.vvb.data

import java.io.File

data class StateSlot(val file: File, val index: Int, val exists: Boolean, val lastModified: Long) {
    constructor(file: File, index: Int): this(file, index, file.exists(), file.lastModified())
}