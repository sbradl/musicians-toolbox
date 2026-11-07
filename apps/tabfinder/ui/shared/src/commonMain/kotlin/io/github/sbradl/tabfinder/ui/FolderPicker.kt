package io.github.sbradl.tabfinder.ui

fun interface FolderPicker {
    suspend fun pickFolder(title: String): FolderPick
}

sealed interface FolderPick {
    data class Picked(
        val location: String,
    ) : FolderPick

    data object Cancelled : FolderPick

    data object Unavailable : FolderPick
}
