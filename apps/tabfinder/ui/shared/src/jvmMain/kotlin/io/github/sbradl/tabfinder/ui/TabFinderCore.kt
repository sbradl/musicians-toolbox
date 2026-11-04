package io.github.sbradl.tabfinder.ui

import io.github.sbradl.tabfinder.core.App
import io.github.sbradl.tabfinder.core.AppViewModel
import io.github.sbradl.tabfinder.core.ViewModelListener
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

class TabFinderCore {
    private val latestViewModel = MutableStateFlow<AppViewModel?>(null)
    val viewModel: StateFlow<AppViewModel?> = latestViewModel.asStateFlow()

    private val app =
        App(
            object : ViewModelListener {
                override fun onViewModel(viewModel: AppViewModel) {
                    latestViewModel.value = viewModel
                }
            },
        )

    fun start() = app.start()

    fun chooseFolder() = app.chooseFolder()
}
