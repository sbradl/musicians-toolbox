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
        val libraryFolderId = libraryFolderIdOf(tree) ?: throw LibraryFolderException.Gone()
        val entries = entriesIn(tree, libraryFolderId) ?: throw LibraryFolderException.Gone()

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

    private fun libraryFolderIdOf(tree: Uri): String? =
        try {
            DocumentsContract.getTreeDocumentId(tree)
        } catch (_: IllegalArgumentException) {
            null
        }

    private fun entriesIn(
        tree: Uri,
        folderId: String,
    ): List<FolderEntry>? {
        val folderContents = DocumentsContract.buildChildDocumentsUriUsingTree(tree, folderId)

        return try {
            queryEntries(folderContents)
        } catch (_: SecurityException) {
            null
        } catch (_: IllegalArgumentException) {
            null
        }
    }

    private fun queryEntries(folderContents: Uri): List<FolderEntry>? {
        val noFilter = null
        val noFilterArguments = null
        val anyOrder = null
        val cursor = contentResolver.query(folderContents, FolderEntry.COLUMNS, noFilter, noFilterArguments, anyOrder) ?: return null

        return cursor.use { entriesOf(cursor) }
    }

    private fun entriesOf(cursor: Cursor): List<FolderEntry> {
        val entries = mutableListOf<FolderEntry>()
        while (cursor.moveToNext()) {
            val entry = FolderEntry.fromCurrentRow(cursor)

            if (entry != null) {
                entries += entry
            }
        }

        return entries
    }
}

private class FolderEntry(
    val id: String,
    val name: String,
    val mimeType: String,
) {
    fun isVisibleSubfolder() = isFolder() && !name.startsWith(".")

    fun isTabFile() = !isFolder() && hasTitle() && extension() in TAB_FILE_EXTENSIONS

    fun nameWithoutExtension() = name.substringBeforeLast('.')

    private fun isFolder() = mimeType == Document.MIME_TYPE_DIR

    private fun hasTitle() = nameWithoutExtension().isNotEmpty()

    private fun extension() = name.substringAfterLast('.', "").lowercase()

    companion object {
        val COLUMNS = arrayOf(Document.COLUMN_DOCUMENT_ID, Document.COLUMN_DISPLAY_NAME, Document.COLUMN_MIME_TYPE)
        private val TAB_FILE_EXTENSIONS = setOf("gp3", "gp4", "gp5", "gpx", "gp", "tg", "ptb")

        fun fromCurrentRow(cursor: Cursor): FolderEntry? {
            val id = cursor.stringInColumn(Document.COLUMN_DOCUMENT_ID) ?: return null
            val name = cursor.stringInColumn(Document.COLUMN_DISPLAY_NAME) ?: return null
            val mimeType = cursor.stringInColumn(Document.COLUMN_MIME_TYPE) ?: return null

            return FolderEntry(id, name, mimeType)
        }

        private fun Cursor.stringInColumn(column: String): String? = getString(getColumnIndexOrThrow(column))
    }
}
