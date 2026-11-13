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
import io.github.sbradl.tabfinder.core.TabFileContents
import io.github.sbradl.tabfinder.core.songFromFile
import java.io.ByteArrayOutputStream
import java.io.IOException
import java.io.InputStream

class SafLibraryFolder(
    private val contentResolver: ContentResolver,
) : LibraryFolder {
    override fun songs(location: String): List<Song> {
        val tree = location.toUri()
        val libraryFolderId = libraryFolderIdOf(tree) ?: throw LibraryFolderException.Gone()
        val entries = libraryFolderEntries(tree, libraryFolderId)

        return songsIn(tree, entries, foldersDownToSong = emptyList())
    }

    private fun libraryFolderEntries(
        tree: Uri,
        libraryFolderId: String,
    ): List<FolderEntry> {
        val folderContents = DocumentsContract.buildChildDocumentsUriUsingTree(tree, libraryFolderId)
        val entries =
            try {
                queryEntries(folderContents)
            } catch (_: SecurityException) {
                throw LibraryFolderException.NoAccess()
            } catch (_: IllegalArgumentException) {
                throw LibraryFolderException.Gone()
            }

        return entries ?: throw LibraryFolderException.Gone()
    }

    private fun songsIn(
        tree: Uri,
        entries: List<FolderEntry>,
        foldersDownToSong: List<String>,
    ): List<Song> =
        entries.flatMap { entry ->
            when {
                entry.isVisibleSubfolder() -> songsInSubfolder(tree, entry, foldersDownToSong)
                entry.isFolder() -> emptyList()
                else -> listOfNotNull(songFromFile(foldersDownToSong, entry.name, DocumentContents(contentResolver, tree, entry.id)))
            }
        }

    private fun songsInSubfolder(
        tree: Uri,
        subfolder: FolderEntry,
        foldersDownToSong: List<String>,
    ): List<Song> {
        val entries = entriesIn(tree, subfolder.id) ?: return emptyList()

        return songsIn(tree, entries, foldersDownToSong + subfolder.name)
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

private class DocumentContents(
    private val contentResolver: ContentResolver,
    private val tree: Uri,
    private val documentId: String,
) : TabFileContents {
    override fun read(mostBytes: ULong): ByteArray? {
        val document = DocumentsContract.buildDocumentUriUsingTree(tree, documentId)

        return try {
            contentResolver.openInputStream(document)?.use { it.readAtMost(mostBytes.toInt()) }
        } catch (_: IOException) {
            null
        } catch (_: SecurityException) {
            null
        }
    }
}

private fun InputStream.readAtMost(mostBytes: Int): ByteArray {
    val contents = ByteArrayOutputStream()
    val chunk = ByteArray(CHUNK_BYTES)
    var read = 0
    while (read < mostBytes) {
        val count = read(chunk, 0, minOf(chunk.size, mostBytes - read))

        if (count < 0) {
            break
        }
        contents.write(chunk, 0, count)
        read += count
    }

    return contents.toByteArray()
}

private const val CHUNK_BYTES = 64 * 1024

private class FolderEntry(
    val id: String,
    val name: String,
    val mimeType: String,
) {
    fun isVisibleSubfolder() = isFolder() && !name.startsWith(".")

    fun isFolder() = mimeType == Document.MIME_TYPE_DIR

    companion object {
        val COLUMNS = arrayOf(Document.COLUMN_DOCUMENT_ID, Document.COLUMN_DISPLAY_NAME, Document.COLUMN_MIME_TYPE)

        fun fromCurrentRow(cursor: Cursor): FolderEntry? {
            val id = cursor.stringInColumn(Document.COLUMN_DOCUMENT_ID) ?: return null
            val name = cursor.stringInColumn(Document.COLUMN_DISPLAY_NAME) ?: return null
            val mimeType = cursor.stringInColumn(Document.COLUMN_MIME_TYPE) ?: return null

            return FolderEntry(id, name, mimeType)
        }

        private fun Cursor.stringInColumn(column: String): String? = getString(getColumnIndexOrThrow(column))
    }
}
