package com.simongellis.vvb.data

import kotlinx.serialization.Serializable

@Serializable
data class Controller(
    override val id: String,
    val name: String,
    val keyMappings: List<KeyMapping>,
    val axisMappings: List<AxisMapping>
): Entity
