package com.simongellis.vvb.emulator

enum class Input(val prefName: String, val bitMask: Int) {
    LL("input_ll", 0x0200),
    LR("input_lr", 0x0100),
    LU("input_lu", 0x0800),
    LD("input_ld", 0x0400),

    RL("input_rl", 0x4000),
    RR("input_rr", 0x0080),
    RU("input_ru", 0x0040),
    RD("input_rd", 0x8000),

    A("input_a", 0x0004),
    B("input_b", 0x0008),
    LT("input_lt", 0x0020),
    RT("input_rt", 0x0010),
    SELECT("input_select", 0x2000),
    START("input_start", 0x1000),
}
