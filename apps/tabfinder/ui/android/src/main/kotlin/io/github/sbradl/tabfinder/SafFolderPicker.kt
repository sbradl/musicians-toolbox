package io.github.sbradl.tabfinder

import androidx.activity.ComponentActivity
import androidx.activity.result.contract.ActivityResultContracts.OpenDocumentTree
import io.github.sbradl.tabfinder.ui.FolderPicker
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

class SafFolderPicker(
    activity: ComponentActivity,
) : FolderPicker {
    private var pickedFolder = CompletableDeferred<String?>()
    private val systemPicker =
        activity.registerForActivityResult(OpenDocumentTree()) { tree ->
            pickedFolder.complete(tree?.toString())
        }

    override suspend fun pickFolder(): String? {
        pickedFolder = CompletableDeferred()
        withContext(Dispatchers.Main) { systemPicker.launch(null) }

        return pickedFolder.await()
    }
}
