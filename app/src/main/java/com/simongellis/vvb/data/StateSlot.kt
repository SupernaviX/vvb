package com.simongellis.vvb.data

import java.io.File

data class StateSlot(val file: File, val name: String, val exists: Boolean, val lastModified: Long) {
    constructor(file: File, name: String): this(file, name, file.exists(), file.lastModified())
}