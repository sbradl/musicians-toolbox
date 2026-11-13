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
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExposedDropdownMenuAnchorType
import androidx.compose.material3.ExposedDropdownMenuBox
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import io.github.sbradl.tabfinder.core.Action
import io.github.sbradl.tabfinder.core.FilterField
import io.github.sbradl.tabfinder.core.Filters
import io.github.sbradl.tabfinder.core.LibraryViewModel
import io.github.sbradl.tabfinder.core.NothingMatches
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
            onArtistFilterChanged = core::artistFilterChanged,
            onSongFilterChanged = core::songFilterChanged,
            onClearFilters = core::clearFilters,
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
    onArtistFilterChanged: (String) -> Unit,
    onSongFilterChanged: (String) -> Unit,
    onClearFilters: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier = modifier.fillMaxSize().padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        Header(viewModel, appName, onChooseFolder, onRescan)
        Problem(viewModel.problem)
        val filters = viewModel.filters

        if (filters != null) {
            FilterBar(filters, onArtistFilterChanged, onSongFilterChanged)
            NoSongsMatch(filters.nothingMatches, onClearFilters)
        }
        SongList(viewModel.songs)
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun FilterBar(
    filters: Filters,
    onArtistFilterChanged: (String) -> Unit,
    onSongFilterChanged: (String) -> Unit,
) {
    FlowRow(
        horizontalArrangement = Arrangement.spacedBy(16.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp),
        itemVerticalAlignment = Alignment.CenterVertically,
    ) {
        FilterTextFieldWithSuggestions(filters.artist, onArtistFilterChanged)
        FilterTextField(filters.song, onSongFilterChanged)
        Text(filters.matchCount)
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun FilterTextFieldWithSuggestions(
    field: FilterField,
    onTextChanged: (String) -> Unit,
) {
    var wantsSuggestions by remember { mutableStateOf(false) }
    val showsSuggestions = wantsSuggestions && field.suggestions.isNotEmpty()
    val onTyped = { text: String ->
        wantsSuggestions = true
        onTextChanged(text)
    }
    ExposedDropdownMenuBox(expanded = showsSuggestions, onExpandedChange = { wantsSuggestions = it }) {
        val anchor =
            Modifier
                .menuAnchor(ExposedDropdownMenuAnchorType.PrimaryEditable)
                .onFocusChanged { focus -> if (!focus.isFocused) wantsSuggestions = false }
        FilterTextField(field, onTyped, anchor)
        ExposedDropdownMenu(expanded = showsSuggestions, onDismissRequest = { wantsSuggestions = false }) {
            field.suggestions.forEach { suggestion ->
                DropdownMenuItem(text = { Text(suggestion) }, onClick = {
                    wantsSuggestions = false
                    onTextChanged(suggestion)
                })
            }
        }
    }
}

@Composable
private fun FilterTextField(
    field: FilterField,
    onTextChanged: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    OutlinedTextField(
        value = field.text,
        onValueChange = onTextChanged,
        label = { Text(field.label) },
        singleLine = true,
        trailingIcon = { ClearButton(field, onTextChanged) },
        modifier = modifier,
    )
}

@Composable
private fun ClearButton(
    field: FilterField,
    onTextChanged: (String) -> Unit,
) {
    if (!field.clear.isEnabled) {
        return
    }

    IconButton(onClick = { onTextChanged("") }, modifier = Modifier.semantics { contentDescription = field.clear.label }) {
        Text("✕", modifier = Modifier.clearAndSetSemantics {})
    }
}

@Composable
private fun NoSongsMatch(
    nothingMatches: NothingMatches?,
    onClearFilters: () -> Unit,
) {
    if (nothingMatches == null) {
        return
    }

    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Text(nothingMatches.text)
        Button(onClick = onClearFilters) {
            Text(nothingMatches.clearFilters)
        }
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
        items(songs) { song -> SongRow(song) }
    }
}

@Composable
private fun SongRow(song: SongItem) {
    Column(modifier = Modifier.padding(vertical = 4.dp)) {
        Text(song.title)
        val artistAndAlbum = song.artistAndAlbum

        if (artistAndAlbum != null) {
            Text(artistAndAlbum, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
    }
}
