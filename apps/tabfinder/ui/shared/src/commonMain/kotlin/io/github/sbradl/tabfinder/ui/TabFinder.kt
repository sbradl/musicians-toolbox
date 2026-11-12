package io.github.sbradl.tabfinder.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import io.github.sbradl.tabfinder.core.Action
import io.github.sbradl.tabfinder.core.LibraryViewModel
import io.github.sbradl.tabfinder.core.SongItem

@Composable
fun TabFinder(
    core: TabFinderCore,
    appName: String,
) {
    val viewModel by core.viewModel.collectAsState()
    val messages = remember { SnackbarHostState() }
    LaunchedEffect(core) { core.messages.collect { message -> messages.showSnackbar(message) } }
    val library = viewModel?.library ?: return
    Scaffold(snackbarHost = { SnackbarHost(messages) }) { insets ->
        LibraryScreen(
            library,
            appName,
            onChooseFolder = core::chooseFolder,
            onRescan = core::rescan,
            modifier = Modifier.padding(insets),
        )
    }
}

@Composable
internal fun LibraryScreen(
    viewModel: LibraryViewModel,
    appName: String,
    onChooseFolder: () -> Unit,
    onRescan: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier = modifier.fillMaxSize().padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        Header(viewModel, appName, onChooseFolder, onRescan)
        Problem(viewModel.problem)
        SongList(viewModel.songs)
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun Header(
    viewModel: LibraryViewModel,
    appName: String,
    onChooseFolder: () -> Unit,
    onRescan: () -> Unit,
) {
    FlowRow(
        horizontalArrangement = Arrangement.spacedBy(16.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        Column {
            Text(appName, style = MaterialTheme.typography.titleLarge)
            Text(viewModel.status)
        }
        ActionButton(viewModel.chooseFolder, onChooseFolder)
        ActionButton(viewModel.rescan, onRescan)
    }
}

@Composable
private fun Problem(problem: String?) {
    if (problem != null) {
        Text(problem, color = MaterialTheme.colorScheme.error)
    }
}

@Composable
private fun ActionButton(
    action: Action,
    onClick: () -> Unit,
) {
    Button(onClick = onClick, enabled = action.isEnabled) {
        Text(action.label)
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
