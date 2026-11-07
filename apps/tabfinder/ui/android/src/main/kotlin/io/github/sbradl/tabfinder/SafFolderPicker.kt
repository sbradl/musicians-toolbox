package io.github.sbradl.tabfinder

import android.content.ContentResolver
import android.content.Intent
import android.net.Uri
import androidx.activity.result.ActivityResultLauncher
import androidx.activity.result.ActivityResultRegistry
import androidx.activity.result.contract.ActivityResultContracts.OpenDocumentTree
import io.github.sbradl.tabfinder.ui.FolderPick
import io.github.sbradl.tabfinder.ui.FolderPicker
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

class SafFolderPicker(
    private val contentResolver: ContentResolver,
) : FolderPicker {
    @Volatile
    private var systemPicker: ActivityResultLauncher<Uri?>? = null

    @Volatile
    private var answer = CompletableDeferred<FolderPick>()

    fun attachTo(screen: ActivityResultRegistry) {
        systemPicker = screen.register(SYSTEM_PICKER_KEY, OpenDocumentTree(), ::answerWith)
    }

    override suspend fun pickFolder(title: String): FolderPick {
        answer = CompletableDeferred()
        openSystemPicker()

        return answer.await()
    }

    private suspend fun openSystemPicker() {
        val picker = checkNotNull(systemPicker) { "SafFolderPicker is not attached to a screen" }
        val noStartingFolder: Uri? = null
        withContext(Dispatchers.Main) { picker.launch(noStartingFolder) }
    }

    private fun answerWith(pickedTree: Uri?) {
        if (pickedTree == null) {
            answer.complete(FolderPick.Cancelled)

            return
        }
        keepAccessAcrossRestarts(pickedTree)
        answer.complete(FolderPick.Picked(pickedTree.toString()))
    }

    private fun keepAccessAcrossRestarts(tree: Uri) {
        contentResolver.takePersistableUriPermission(tree, Intent.FLAG_GRANT_READ_URI_PERMISSION)
    }

    private companion object {
        const val SYSTEM_PICKER_KEY = "tabfinder-library-folder-picker"
    }
}
