package io.github.sbradl.tabfinder.ui

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assertIsNotEnabled
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.runComposeUiTest
import io.github.sbradl.tabfinder.core.Action
import io.github.sbradl.tabfinder.core.LibraryViewModel
import io.github.sbradl.tabfinder.ui.support.runTabFinderTest
import kotlin.test.Test

class LibraryScreenTest {
    @Test
    fun noFolderChosenOffersChooseFolder() =
        runTabFinderTest { tabfinder ->
            tabfinder.musicianOpensTabFinder()

            tabfinder.assertShown("No folder chosen", "Choose folder")
        }

    @Test
    fun choosingAFolderWithTabFilesShowsItsSongs() =
        runTabFinderTest { tabfinder ->
            tabfinder.libraryFolderContains("Wexmoor.gp5", "Some Artist/Brasswing.gp5", "notes.txt")
            tabfinder.musicianOpensTabFinder()

            tabfinder.musicianPicksTheLibraryFolder()

            tabfinder.assertStatus("2 Songs")
            tabfinder.assertShown("Brasswing", "Wexmoor")
        }

    @Test
    fun choosingAFolderWithoutTabFilesShowsNoTabFilesFound() =
        runTabFinderTest { tabfinder ->
            tabfinder.libraryFolderContains("notes.txt")
            tabfinder.musicianOpensTabFinder()

            tabfinder.musicianPicksTheLibraryFolder()

            tabfinder.assertStatus("No Tab files found")
        }

    @OptIn(ExperimentalTestApi::class)
    @Test
    fun scanningShowsItsStatusAndDisablesChooseFolder() =
        runComposeUiTest {
            setContent {
                LibraryScreen(
                    LibraryViewModel(
                        status = "Scanning…",
                        chooseFolder = Action(label = "Choose folder", isEnabled = false),
                        songs = emptyList(),
                    ),
                    onChooseFolder = {},
                )
            }

            onNodeWithText("Scanning…").assertExists()
            onNodeWithText("Choose folder").assertIsNotEnabled()
        }

    @Test
    fun everyInteractiveElementHasALabel() =
        runTabFinderTest { tabfinder ->
            tabfinder.libraryFolderContains("Wexmoor.gp5")
            tabfinder.musicianOpensTabFinder()
            tabfinder.musicianPicksTheLibraryFolder()
            tabfinder.assertStatus("1 Song")

            tabfinder.assertEveryInteractiveElementIsLabelled()
        }
}
