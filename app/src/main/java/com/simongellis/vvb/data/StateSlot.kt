package com.simongellis.vvb.data

import java.io.File

data class StateSlot(val file: File, val name: String, val exists: Boolean, val lastModified: Long) {
    constructor(file: File): this(file, file.nameWithoutExtension, file.exists(), file.lastModified())
}