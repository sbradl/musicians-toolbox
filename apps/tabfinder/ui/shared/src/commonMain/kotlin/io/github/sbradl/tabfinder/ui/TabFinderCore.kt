package io.github.sbradl.tabfinder.ui

import io.github.sbradl.tabfinder.core.App
import io.github.sbradl.tabfinder.core.AppViewModel
import io.github.sbradl.tabfinder.core.Effect
import io.github.sbradl.tabfinder.core.LibraryFolder
import io.github.sbradl.tabfinder.core.ViewModelListener
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

class TabFinderCore private constructor(
    private val folderPicker: FolderPicker,
    connectApp: (ViewModelListener) -> App,
) {
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
    private val shownViewModel = MutableStateFlow<AppViewModel?>(null)
    val viewModel: StateFlow<AppViewModel?> = shownViewModel.asStateFlow()

    private val app = connectApp(ListenerForRust())

    fun start() = app.start()

    fun chooseFolder() = app.chooseFolder()

    private fun show(viewModel: AppViewModel) {
        shownViewModel.value = viewModel
    }

    private fun perform(effect: Effect) =
        when (effect) {
            Effect.PICK_FOLDER -> pickFolder()
        }

    private fun pickFolder() {
        scope.launch {
            val pickedFolder = folderPicker.pickFolder()

            if (pickedFolder != null) {
                scanOffTheUiThread { app.folderPicked(pickedFolder) }
            }
        }
    }

    private suspend fun scanOffTheUiThread(scan: () -> Unit) = withContext(Dispatchers.IO) { scan() }

    companion object {
        fun onDesktop(
            folderPicker: FolderPicker,
            configDir: String,
        ) = TabFinderCore(folderPicker) { listener -> App(listener, configDir) }

        fun onAndroid(
            folderPicker: FolderPicker,
            configDir: String,
            libraryFolder: LibraryFolder,
        ) = TabFinderCore(folderPicker) { listener -> App.withLibraryFolder(listener, configDir, libraryFolder) }
    }

    private inner class ListenerForRust : ViewModelListener {
        override fun onViewModel(viewModel: AppViewModel) = show(viewModel)

        override fun onEffect(effect: Effect) = perform(effect)
    }
}
