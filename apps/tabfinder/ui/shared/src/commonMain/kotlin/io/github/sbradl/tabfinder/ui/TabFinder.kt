package io.github.sbradl.tabfinder.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import io.github.sbradl.tabfinder.core.Action
import io.github.sbradl.tabfinder.core.LibraryViewModel
import io.github.sbradl.tabfinder.core.SongItem

@Composable
fun TabFinder(core: TabFinderCore) {
    val viewModel by core.viewModel.collectAsState()
    val library = viewModel?.library ?: return
    LibraryScreen(library, onChooseFolder = core::chooseFolder)
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
        Problem(viewModel.problem)
        ChooseFolderButton(viewModel.chooseFolder, onChooseFolder)
        SongList(viewModel.songs)
    }
}

@Composable
private fun Problem(problem: String?) {
    if (problem != null) {
        Text(problem, color = MaterialTheme.colorScheme.error)
    }
}

@Composable
private fun ChooseFolderButton(
    chooseFolder: Action,
    onClick: () -> Unit,
) {
    Button(onClick = onClick, enabled = chooseFolder.isEnabled) {
        Text(chooseFolder.label)
    }
}

@Composable
private fun SongList(songs: List<SongItem>) {
    LazyColumn(modifier = Modifier.fillMaxWidth()) {
        items(songs) { song ->
            Text(song.title, modifier = Modifier.padding(vertical = 4.dp))
        }
    }
}
