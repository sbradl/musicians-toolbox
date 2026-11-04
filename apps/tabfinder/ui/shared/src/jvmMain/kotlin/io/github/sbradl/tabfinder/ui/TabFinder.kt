package io.github.sbradl.tabfinder.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.Button
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import io.github.sbradl.tabfinder.core.LibraryViewModel

@Composable
fun TabFinder(
    core: TabFinderCore,
    onChooseFolder: () -> Unit = core::chooseFolder,
) {
    val viewModel by core.viewModel.collectAsState()
    viewModel?.let { LibraryScreen(it.library, onChooseFolder) }
}

@Composable
private fun LibraryScreen(
    viewModel: LibraryViewModel,
    onChooseFolder: () -> Unit,
) {
    Column(
        modifier = Modifier.fillMaxSize(),
        verticalArrangement = Arrangement.spacedBy(16.dp, Alignment.CenterVertically),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Text(viewModel.status)
        Button(
            onClick = onChooseFolder,
            enabled = viewModel.chooseFolder.isEnabled,
        ) {
            Text(viewModel.chooseFolder.label)
        }
    }
}
