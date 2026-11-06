package io.github.sbradl.tabfinder

import android.content.ContentResolver
import android.database.Cursor
import android.net.Uri
import android.provider.DocumentsContract
import android.provider.DocumentsContract.Document
import androidx.core.net.toUri
import io.github.sbradl.tabfinder.core.LibraryFolder
import io.github.sbradl.tabfinder.core.LibraryFolderException
import io.github.sbradl.tabfinder.core.Song

class SafLibraryFolder(
    private val contentResolver: ContentResolver,
) : LibraryFolder {
    override fun songs(location: String): List<Song> {
        val tree = location.toUri()
        val libraryFolder = rootFolderOf(tree) ?: throw LibraryFolderException.Gone()
        val entries = entriesIn(tree, libraryFolder) ?: throw LibraryFolderException.Gone()

        return songsIn(tree, entries)
    }

    private fun songsIn(
        tree: Uri,
        entries: List<FolderEntry>,
    ): List<Song> =
        entries.flatMap { entry ->
            when {
                entry.isVisibleSubfolder() -> songsInSubfolder(tree, entry)
                entry.isTabFile() -> listOf(Song(title = entry.nameWithoutExtension()))
                else -> emptyList()
            }
        }

    private fun songsInSubfolder(
        tree: Uri,
        subfolder: FolderEntry,
    ): List<Song> {
        val entries = entriesIn(tree, subfolder.id) ?: return emptyList()

        return songsIn(tree, entries)
    }

    private fun rootFolderOf(tree: Uri): String? =
        try {
            DocumentsContract.getTreeDocumentId(tree)
        } catch (_: IllegalArgumentException) {
            null
        }

    private fun entriesIn(
        tree: Uri,
        folderId: String,
    ): List<FolderEntry>? {
        val children = DocumentsContract.buildChildDocumentsUriUsingTree(tree, folderId)

        return try {
            contentResolver.query(children, FolderEntry.COLUMNS, null, null, null)?.use(::entriesOf)
        } catch (_: SecurityException) {
            null
        } catch (_: IllegalArgumentException) {
            null
        }
    }

    private fun entriesOf(cursor: Cursor): List<FolderEntry> {
        val entries = mutableListOf<FolderEntry>()
        while (cursor.moveToNext()) {
            entries += FolderEntry.at(cursor) ?: continue
        }

        return entries
    }
}

private class FolderEntry(
    val id: String,
    val name: String,
    val mimeType: String,
) {
    fun isVisibleSubfolder() = mimeType == Document.MIME_TYPE_DIR && !name.startsWith(".")

    fun isTabFile() = mimeType != Document.MIME_TYPE_DIR && nameWithoutExtension().isNotEmpty() && extension() in TAB_FILE_EXTENSIONS

    fun nameWithoutExtension() = name.substringBeforeLast('.')

    private fun extension() = name.substringAfterLast('.', "").lowercase()

    companion object {
        val COLUMNS = arrayOf(Document.COLUMN_DOCUMENT_ID, Document.COLUMN_DISPLAY_NAME, Document.COLUMN_MIME_TYPE)
        private val TAB_FILE_EXTENSIONS = setOf("gp3", "gp4", "gp5", "gpx", "gp", "tg", "ptb")

        fun at(cursor: Cursor): FolderEntry? {
            val id = cursor.stringIn(Document.COLUMN_DOCUMENT_ID) ?: return null
            val name = cursor.stringIn(Document.COLUMN_DISPLAY_NAME) ?: return null
            val mimeType = cursor.stringIn(Document.COLUMN_MIME_TYPE) ?: return null

            return FolderEntry(id, name, mimeType)
        }

        private fun Cursor.stringIn(column: String): String? = getString(getColumnIndexOrThrow(column))
    }
}
