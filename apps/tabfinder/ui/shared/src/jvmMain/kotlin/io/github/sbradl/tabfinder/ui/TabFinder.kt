package io.github.sbradl.tabfinder.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
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
fun TabFinder(core: TabFinderCore) {
    val viewModel by core.viewModel.collectAsState()
    viewModel?.let { LibraryScreen(it.library, core::chooseFolder) }
}

@Composable
internal fun LibraryScreen(
    viewModel: LibraryViewModel,
    onChooseFolder: () -> Unit,
) {
    Column(
        modifier = Modifier.fillMaxSize().padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Text(viewModel.status)
        Button(
            onClick = onChooseFolder,
            enabled = viewModel.chooseFolder.isEnabled,
        ) {
            Text(viewModel.chooseFolder.label)
        }
        LazyColumn(modifier = Modifier.fillMaxWidth()) {
            items(viewModel.songs, key = { it.id }) { song ->
                Text(song.title, modifier = Modifier.padding(vertical = 4.dp))
            }
        }
    }
}
