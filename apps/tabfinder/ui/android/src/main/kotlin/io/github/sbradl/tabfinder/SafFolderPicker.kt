package io.github.sbradl.tabfinder

import android.net.Uri
import androidx.activity.ComponentActivity
import androidx.activity.result.contract.ActivityResultContracts.OpenDocumentTree
import io.github.sbradl.tabfinder.ui.FolderPicker
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

class SafFolderPicker(
    activity: ComponentActivity,
) : FolderPicker {
    private val systemPicker = activity.registerForActivityResult(OpenDocumentTree(), ::answerWith)
    private var answer = CompletableDeferred<String?>()

    override suspend fun pickFolder(): String? {
        answer = CompletableDeferred()
        openSystemPicker()

        return answer.await()
    }

    private suspend fun openSystemPicker() {
        val noStartingFolder: Uri? = null
        withContext(Dispatchers.Main) { systemPicker.launch(noStartingFolder) }
    }

    private fun answerWith(pickedTree: Uri?) {
        answer.complete(pickedTree?.toString())
    }
}
