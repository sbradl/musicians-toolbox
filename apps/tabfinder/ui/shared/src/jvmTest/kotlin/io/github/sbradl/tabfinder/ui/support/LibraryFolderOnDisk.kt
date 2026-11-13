package io.github.sbradl.tabfinder.ui.support

import io.github.sbradl.tabfinder.core.LibraryFolder
import io.github.sbradl.tabfinder.core.Song
import io.github.sbradl.tabfinder.core.songFromFile
import java.io.File
import java.nio.file.Path

class LibraryFolderOnDisk(
    private val folder: Path,
) : LibraryFolder {
    override fun songs(location: String): List<Song> {
        val files = folder.toFile().walk().filter { it.isFile }

        return files.mapNotNull { file -> songFromFile(foldersDownTo(file), file.name) }.toList()
    }

    private fun foldersDownTo(file: File) =
        file.parentFile.relativeTo(folder.toFile()).invariantSeparatorsPath.split("/").filter {
            it.isNotEmpty()
        }
}
