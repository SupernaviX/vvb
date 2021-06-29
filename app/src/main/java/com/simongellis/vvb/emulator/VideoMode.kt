package com.simongellis.vvb.emulator

enum class VideoMode(val supportsPortait: Boolean) {
    ANAGLYPH(true),
    CARDBOARD(false),
    MONO_LEFT(true),
    MONO_RIGHT(true),
    STEREO(false);
}
