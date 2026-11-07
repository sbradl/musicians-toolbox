package io.github.sbradl.tabfinder

import android.content.Intent
import android.net.Uri
import androidx.activity.ComponentActivity
import androidx.activity.result.contract.ActivityResultContracts.OpenDocumentTree
import io.github.sbradl.tabfinder.ui.FolderPicker
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

class SafFolderPicker(
    private val activity: ComponentActivity,
) : FolderPicker {
    private val systemPicker = activity.registerForActivityResult(OpenDocumentTree(), ::answerWith)
    private var answer = CompletableDeferred<String?>()

    override suspend fun pickFolder(title: String): String? {
        answer = CompletableDeferred()
        openSystemPicker()

        return answer.await()
    }

    private suspend fun openSystemPicker() {
        val noStartingFolder: Uri? = null
        withContext(Dispatchers.Main) { systemPicker.launch(noStartingFolder) }
    }

    private fun answerWith(pickedTree: Uri?) {
        if (pickedTree != null) {
            keepAccessAcrossRestarts(pickedTree)
        }
        answer.complete(pickedTree?.toString())
    }

    private fun keepAccessAcrossRestarts(tree: Uri) {
        activity.contentResolver.takePersistableUriPermission(tree, Intent.FLAG_GRANT_READ_URI_PERMISSION)
    }
}
