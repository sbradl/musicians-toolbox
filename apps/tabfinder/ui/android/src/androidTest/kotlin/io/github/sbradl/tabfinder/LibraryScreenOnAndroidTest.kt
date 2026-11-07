package io.github.sbradl.tabfinder

import io.github.sbradl.tabfinder.support.runTabFinderTest
import kotlin.test.Test

class LibraryScreenOnAndroidTest {
    @Test
    fun noFolderChosenOffersChooseFolder() =
        runTabFinderTest { tabfinder ->
            tabfinder.musicianOpensTabFinder()

            tabfinder.assertShown("No folder chosen", "Choose folder")
        }

    @Test
    fun tabFilesInSubfoldersAreFoundIgnoringTheCaseOfTheirExtension() =
        runTabFinderTest { tabfinder ->
            tabfinder.libraryFolderContains("Wexmoor.GP5", "Some Artist/Brasswing.tg", "notes.txt")
            tabfinder.musicianOpensTabFinder()

            tabfinder.musicianPicksTheLibraryFolder()

            tabfinder.assertStatus("2 Songs")
            tabfinder.assertShown("Brasswing", "Wexmoor")
        }

    @Test
    fun tabFilesInHiddenFoldersAreSkipped() =
        runTabFinderTest { tabfinder ->
            tabfinder.libraryFolderContains("Wexmoor.gp5", ".backup/Old Wexmoor.gp5")
            tabfinder.musicianOpensTabFinder()

            tabfinder.musicianPicksTheLibraryFolder()

            tabfinder.assertStatus("1 Song")
            tabfinder.assertNotShown("Old Wexmoor")
        }

    @Test
    fun hiddenFilesAreNoSongs() =
        runTabFinderTest { tabfinder ->
            tabfinder.libraryFolderContains("Wexmoor.gp5", ".Old Wexmoor.gp5")
            tabfinder.musicianOpensTabFinder()

            tabfinder.musicianPicksTheLibraryFolder()

            tabfinder.assertStatus("1 Song")
            tabfinder.assertNotShown(".Old Wexmoor")
        }

    @Test
    fun aFileNamedOnlyByAnExtensionIsNoSong() =
        runTabFinderTest { tabfinder ->
            tabfinder.libraryFolderContains(".gp5")
            tabfinder.musicianOpensTabFinder()

            tabfinder.musicianPicksTheLibraryFolder()

            tabfinder.assertStatus("No Tab files found")
        }

    @Test
    fun aFolderWithoutTabFilesShowsNoTabFilesFound() =
        runTabFinderTest { tabfinder ->
            tabfinder.libraryFolderContains("notes.txt")
            tabfinder.musicianOpensTabFinder()

            tabfinder.musicianPicksTheLibraryFolder()

            tabfinder.assertStatus("No Tab files found")
        }

    @Test
    fun reopeningTabFinderShowsTheRememberedLibrary() =
        runTabFinderTest { tabfinder ->
            tabfinder.libraryFolderContains("Wexmoor.gp5")
            tabfinder.musicianOpensTabFinder()
            tabfinder.musicianPicksTheLibraryFolder()
            tabfinder.assertStatus("1 Song")

            tabfinder.musicianReopensTabFinder()

            tabfinder.assertStatus("1 Song")
            tabfinder.assertShown("Wexmoor")
        }

    @Test
    fun reopeningTabFinderAfterTheLibraryFolderIsGoneShowsNoFolderChosen() =
        runTabFinderTest { tabfinder ->
            tabfinder.libraryFolderContains("Wexmoor.gp5")
            tabfinder.musicianOpensTabFinder()
            tabfinder.musicianPicksTheLibraryFolder()
            tabfinder.assertStatus("1 Song")
            tabfinder.libraryFolderIsDeleted()

            tabfinder.musicianReopensTabFinder()

            tabfinder.assertStatus("No folder chosen")
        }

    @Test
    fun reopeningTabFinderShowsScanningWhileTheRememberedLibraryIsScanned() =
        runTabFinderTest { tabfinder ->
            tabfinder.libraryFolderContains("Wexmoor.gp5")
            tabfinder.musicianOpensTabFinder()
            tabfinder.musicianPicksTheLibraryFolder()
            tabfinder.assertStatus("1 Song")
            tabfinder.libraryFolderIsSlowToRead()

            tabfinder.musicianReopensTabFinder()

            tabfinder.assertStatus("Scanning…")
            tabfinder.libraryFolderFinishesReading()
            tabfinder.assertStatus("1 Song")
        }
}
