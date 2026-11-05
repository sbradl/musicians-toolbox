package io.github.sbradl.tabfinder.ui

import io.github.sbradl.tabfinder.core.App
import io.github.sbradl.tabfinder.core.AppViewModel
import io.github.sbradl.tabfinder.core.Effect
import io.github.sbradl.tabfinder.core.ViewModelListener
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

class TabFinderCore(
    private val folderPicker: FolderPicker,
    configDir: String,
    private val scope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.Default),
) {
    private val latestViewModel = MutableStateFlow<AppViewModel?>(null)
    val viewModel: StateFlow<AppViewModel?> = latestViewModel.asStateFlow()

    private val app =
        App(
            object : ViewModelListener {
                override fun onViewModel(viewModel: AppViewModel) {
                    latestViewModel.value = viewModel
                }

                override fun onEffect(effect: Effect) {
                    when (effect) {
                        Effect.PICK_FOLDER -> pickFolder()
                    }
                }
            },
            configDir,
        )

    fun start() = app.start()

    fun chooseFolder() = app.chooseFolder()

    private fun pickFolder() {
        scope.launch {
            val location = folderPicker.pickFolder()
            withContext(Dispatchers.IO) {
                if (location == null) app.folderPickCancelled() else app.folderPicked(location)
            }
        }
    }
}
