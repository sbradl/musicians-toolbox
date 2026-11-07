package io.github.sbradl.tabfinder

import android.database.Cursor
import android.database.MatrixCursor
import android.os.CancellationSignal
import android.os.ParcelFileDescriptor
import android.provider.DocumentsContract.Document
import android.provider.DocumentsProvider
import java.io.File
import java.io.FileNotFoundException

class LibraryFolderForTests : DocumentsProvider() {
    private val folders: File by lazy { requireNotNull(context).cacheDir }

    override fun onCreate() = true

    override fun queryRoots(projection: Array<out String>?): Cursor = MatrixCursor(projection ?: arrayOf())

    override fun queryDocument(
        documentId: String,
        projection: Array<out String>?,
    ): Cursor = documentsCursor(listOf(fileOf(documentId)))

    override fun queryChildDocuments(
        parentDocumentId: String,
        projection: Array<out String>?,
        sortOrder: String?,
    ): Cursor {
        if (parentDocumentId == FOLDER_IT_MAY_NOT_READ) throw SecurityException("No grant for $parentDocumentId")
        val files = fileOf(parentDocumentId).listFiles() ?: throw FileNotFoundException(parentDocumentId)

        return documentsCursor(files.toList())
    }

    override fun isChildDocument(
        parentDocumentId: String,
        documentId: String,
    ) = documentId.startsWith("$parentDocumentId/")

    override fun openDocument(
        documentId: String,
        mode: String,
        signal: CancellationSignal?,
    ): ParcelFileDescriptor = ParcelFileDescriptor.open(fileOf(documentId), ParcelFileDescriptor.MODE_READ_ONLY)

    private fun fileOf(documentId: String) = File(folders, documentId)

    private fun documentsCursor(files: List<File>): Cursor {
        val cursor = MatrixCursor(arrayOf(Document.COLUMN_DOCUMENT_ID, Document.COLUMN_DISPLAY_NAME, Document.COLUMN_MIME_TYPE))
        files.forEach { file -> cursor.addRow(arrayOf(documentIdOf(file), file.name, mimeTypeOf(file))) }

        return cursor
    }

    private fun documentIdOf(file: File) = file.relativeTo(folders).path

    private fun mimeTypeOf(file: File) = if (file.isDirectory) Document.MIME_TYPE_DIR else "application/octet-stream"

    companion object {
        const val AUTHORITY = "io.github.sbradl.tabfinder.library-folder-for-tests"
        const val FOLDER_IT_MAY_NOT_READ = "locked"
    }
}
