package com.simongellis.vvb.game

import androidx.annotation.StringRes
import com.simongellis.vvb.R

enum class VideoMode(
    val supportsPortrait: Boolean,
    @StringRes val summary: Int,
    @StringRes val description: Int
) {
    LEIA(true, R.string.video_mode_leia_summary, R.string.video_mode_leia_description),
    ANAGLYPH(true, R.string.video_mode_anaglyph_summary, R.string.video_mode_anaglyph_description),
    CARDBOARD(false, R.string.video_mode_cardboard_summary, R.string.video_mode_cardboard_description),
    MONO_LEFT(true, R.string.video_mode_mono_left_summary, R.string.video_mode_mono_description),
    MONO_RIGHT(true, R.string.video_mode_mono_right_summary, R.string.video_mode_mono_description),
    STEREO(false, R.string.video_mode_stereo_summary, R.string.video_mode_stereo_description);
}
