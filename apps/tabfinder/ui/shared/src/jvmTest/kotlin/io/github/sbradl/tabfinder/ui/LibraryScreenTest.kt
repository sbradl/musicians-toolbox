package io.github.sbradl.tabfinder.ui

import androidx.compose.ui.input.key.Key
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

    @Test
    fun scanningShowsItsStatusAndDisablesChooseFolder() =
        runTabFinderTest { tabfinder ->
            tabfinder.libraryFolderContains("Wexmoor.gp5")
            tabfinder.libraryFolderIsSlowToRead()
            tabfinder.musicianOpensTabFinder()

            tabfinder.musicianPicksTheLibraryFolder()

            tabfinder.assertStatus("Scanning…")
            tabfinder.assertChooseFolderIsDisabled()
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

    @Test
    fun theKeyboardAloneChoosesAFolderWithEnter() =
        runTabFinderTest { tabfinder ->
            tabfinder.libraryFolderContains("Wexmoor.gp5")
            tabfinder.musicianOpensTabFinder()

            tabfinder.musicianPicksTheLibraryFolderWithTheKeyboard(Key.Enter)

            tabfinder.assertStatus("1 Song")
        }

    @Test
    fun theKeyboardAloneChoosesAFolderWithSpace() =
        runTabFinderTest { tabfinder ->
            tabfinder.libraryFolderContains("Wexmoor.gp5")
            tabfinder.musicianOpensTabFinder()

            tabfinder.musicianPicksTheLibraryFolderWithTheKeyboard(Key.Spacebar)

            tabfinder.assertStatus("1 Song")
        }

    @Test
    fun tabReachesEveryControl() =
        runTabFinderTest { tabfinder ->
            tabfinder.libraryFolderContains("Wexmoor.gp5")
            tabfinder.musicianOpensTabFinder()
            tabfinder.musicianPicksTheLibraryFolder()
            tabfinder.assertStatus("1 Song")

            tabfinder.assertTabReachesEveryControl()
        }

    @Test
    fun songsAtDoubleTextSizeAreNeitherCutOffNorOverlapping() =
        runTabFinderTest { tabfinder ->
            tabfinder.libraryFolderContains("The Long Ballad of the Brasswing Lighthouse Keeper and the Wexmoor Tide.gp5", "Wexmoor.gp5")
            tabfinder.musicianDoublesTheTextSize()
            tabfinder.musicianOpensTabFinder()

            tabfinder.musicianPicksTheLibraryFolder()

            tabfinder.assertStatus("2 Songs")
            tabfinder.assertNoTextIsCutOffOrOverlapping()
        }

    @Test
    fun aProblemAtDoubleTextSizeIsNeitherCutOffNorOverlapping() =
        runTabFinderTest { tabfinder ->
            tabfinder.musicianDoublesTheTextSize()
            tabfinder.musicianOpensTabFinder()
            tabfinder.libraryFolderIsDeleted()

            tabfinder.musicianPicksTheLibraryFolder()

            tabfinder.assertShown("The Library folder is gone. It may have been moved or deleted.")
            tabfinder.assertNoTextIsCutOffOrOverlapping()
        }
}
