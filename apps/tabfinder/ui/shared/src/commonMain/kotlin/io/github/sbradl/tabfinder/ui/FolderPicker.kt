package io.github.sbradl.tabfinder.ui

fun interface FolderPicker {
    suspend fun pickFolder(title: String): String?
}
