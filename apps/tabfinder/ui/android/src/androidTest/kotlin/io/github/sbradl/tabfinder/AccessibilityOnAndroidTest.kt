package io.github.sbradl.tabfinder

import io.github.sbradl.tabfinder.support.runTabFinderTest
import kotlin.test.Test

class AccessibilityOnAndroidTest {
    @Test
    fun everyControlHasALabel() =
        runTabFinderTest { tabfinder ->
            tabfinder.libraryFolderContains("Wexmoor.gp5")
            tabfinder.musicianOpensTabFinder()
            tabfinder.musicianPicksTheLibraryFolder()
            tabfinder.assertStatus("1 Song")

            tabfinder.assertEveryControlIsLabelled()
        }

    @Test
    fun theKeyboardAloneChoosesAFolder() =
        runTabFinderTest { tabfinder ->
            tabfinder.libraryFolderContains("Wexmoor.gp5")
            tabfinder.musicianOpensTabFinder()

            tabfinder.musicianPicksTheLibraryFolderWithTheKeyboard()

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

            tabfinder.musicianPicksAFolderTabFinderMayNotRead()

            tabfinder.assertShown("TabFinder may not read the Library folder.")
            tabfinder.assertNoTextIsCutOffOrOverlapping()
        }
}
