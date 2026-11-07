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
    fun choosingAFolderShowsItsSongs() =
        runTabFinderTest { tabfinder ->
            tabfinder.libraryFolderContains("Wexmoor.gp5", "Some Artist/Brasswing.gp5", "notes.txt")
            tabfinder.musicianOpensTabFinder()

            tabfinder.musicianPicksTheLibraryFolder()

            tabfinder.assertStatus("2 Songs")
            tabfinder.assertShown("Brasswing", "Wexmoor")
        }

    @Test
    fun choosingAFolderWithoutSongsShowsNoSongsFound() =
        runTabFinderTest { tabfinder ->
            tabfinder.libraryFolderContains("notes.txt")
            tabfinder.musicianOpensTabFinder()

            tabfinder.musicianPicksTheLibraryFolder()

            tabfinder.assertStatus("No Songs found")
        }

    @Test
    fun choosingAFolderThatIsGoneSaysWhy() =
        runTabFinderTest { tabfinder ->
            tabfinder.musicianOpensTabFinder()
            tabfinder.libraryFolderIsDeleted()

            tabfinder.musicianPicksTheLibraryFolder()

            tabfinder.assertStatus("No folder chosen")
            tabfinder.assertShown("The Library folder is gone. It may have been moved or deleted.")
        }

    @Test
    fun withoutAFolderDialogChooseFolderSaysWhy() =
        runTabFinderTest { tabfinder ->
            tabfinder.musicianOpensTabFinder()

            tabfinder.musicianFindsNoFolderDialog()

            tabfinder.assertStatus("No folder chosen")
            tabfinder.assertShown("The system folder dialog is not available.")
        }

    @OptIn(ExperimentalTestApi::class)
    @Test
    fun scanningShowsItsStatusAndDisablesChooseFolder() =
        runComposeUiTest {
            setContent {
                LibraryScreen(
                    LibraryViewModel(
                        status = "Scanning…",
                        problem = null,
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
