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
            tabfinder.libraryFolderContains("Wonderwall.GP5", "Beatles/Blackbird.tg", "notes.txt")
            tabfinder.musicianOpensTabFinder()

            tabfinder.musicianPicksTheLibraryFolder()

            tabfinder.assertStatus("2 Songs")
            tabfinder.assertShown("Blackbird", "Wonderwall")
        }

    @Test
    fun tabFilesInHiddenFoldersAreSkipped() =
        runTabFinderTest { tabfinder ->
            tabfinder.libraryFolderContains("Wonderwall.gp5", ".backup/Old Wonderwall.gp5")
            tabfinder.musicianOpensTabFinder()

            tabfinder.musicianPicksTheLibraryFolder()

            tabfinder.assertStatus("1 Song")
            tabfinder.assertNotShown("Old Wonderwall")
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
}
