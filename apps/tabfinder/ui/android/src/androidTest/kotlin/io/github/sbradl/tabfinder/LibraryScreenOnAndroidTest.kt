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
    fun songsInSubfoldersAreFoundIgnoringTheCaseOfTheirExtension() =
        runTabFinderTest { tabfinder ->
            tabfinder.libraryFolderContains("Wexmoor.GP5", "Some Artist/Brasswing.tg", "notes.txt")
            tabfinder.musicianOpensTabFinder()

            tabfinder.musicianPicksTheLibraryFolder()

            tabfinder.assertStatus("2 Songs")
            tabfinder.assertShown("Brasswing", "Wexmoor")
        }

    @Test
    fun eachSongShowsTheArtistAndAlbumItsFoldersName() =
        runTabFinderTest { tabfinder ->
            tabfinder.libraryFolderContains("Corvid Lane/Saltglass/Wexmoor.gp5", "Hollow Fern/Brasswing.gp5")
            tabfinder.musicianOpensTabFinder()

            tabfinder.musicianPicksTheLibraryFolder()

            tabfinder.assertStatus("2 Songs")
            tabfinder.assertShown("Corvid Lane · Saltglass", "Hollow Fern")
        }

    @Test
    fun theHeaderShowsTabFinderAboveTheSongCount() =
        runTabFinderTest { tabfinder ->
            tabfinder.libraryFolderContains("Wexmoor.gp5")
            tabfinder.musicianOpensTabFinder()

            tabfinder.musicianPicksTheLibraryFolder()

            tabfinder.assertShown("TabFinder", "1 Song")
        }

    @Test
    fun rescanListsNewSongsAndTellsWhatItFound() =
        runTabFinderTest { tabfinder ->
            tabfinder.libraryFolderContains("Wexmoor.gp5")
            tabfinder.musicianOpensTabFinder()
            tabfinder.musicianPicksTheLibraryFolder()
            tabfinder.assertStatus("1 Song")
            tabfinder.libraryFolderContains("Brasswing.gp5")

            tabfinder.musicianClicksRescan()

            tabfinder.assertStatus("2 Songs")
            tabfinder.assertShown("Brasswing", "Wexmoor", "2 Songs, 0 unreadable")
        }

    @Test
    fun typingInTheSongFieldListsOnlySongsWhoseTitleContainsIt() =
        runTabFinderTest { tabfinder ->
            tabfinder.libraryFolderContains("Raise Your Lanterns.gp5", "Wexmoor.gp5")
            tabfinder.musicianOpensTabFinder()
            tabfinder.musicianPicksTheLibraryFolder()
            tabfinder.assertStatus("2 Songs")

            tabfinder.musicianTypesInTheSongField("LANT")

            tabfinder.assertShown("Raise Your Lanterns", "1 / 2")
            tabfinder.assertNotShown("Wexmoor")
        }

    @Test
    fun theCrossEmptiesTheSongFieldAndListsAllSongsAgain() =
        runTabFinderTest { tabfinder ->
            tabfinder.libraryFolderContains("Raise Your Lanterns.gp5", "Wexmoor.gp5")
            tabfinder.musicianOpensTabFinder()
            tabfinder.musicianPicksTheLibraryFolder()
            tabfinder.musicianTypesInTheSongField("lant")
            tabfinder.assertNotShown("Wexmoor")

            tabfinder.musicianClearsTheSongField()

            tabfinder.assertShown("Raise Your Lanterns", "Wexmoor", "2 / 2")
        }

    @Test
    fun whenNoSongMatchesClearFiltersListsAllSongsAgain() =
        runTabFinderTest { tabfinder ->
            tabfinder.libraryFolderContains("Wexmoor.gp5")
            tabfinder.musicianOpensTabFinder()
            tabfinder.musicianPicksTheLibraryFolder()
            tabfinder.musicianTypesInTheSongField("lant")
            tabfinder.assertShown("No Songs match")

            tabfinder.musicianClicksClearFilters()

            tabfinder.assertShown("Wexmoor", "1 / 1")
            tabfinder.assertNotShown("No Songs match")
        }

    @Test
    fun typingInTheArtistFieldListsOnlyThatArtistsSongs() =
        runTabFinderTest { tabfinder ->
            tabfinder.libraryFolderContains("Corvid Lane/Wexmoor.gp5", "Hollow Fern/Brasswing.gp5")
            tabfinder.musicianOpensTabFinder()
            tabfinder.musicianPicksTheLibraryFolder()
            tabfinder.assertStatus("2 Songs")

            tabfinder.musicianTypesInTheArtistField("CORV")

            tabfinder.assertShown("Wexmoor", "1 / 2")
            tabfinder.assertNotShown("Brasswing")
        }

    @Test
    fun pickingAnArtistSuggestionFillsTheArtistField() =
        runTabFinderTest { tabfinder ->
            tabfinder.libraryFolderContains("Corvid Lane/Wexmoor.gp5", "Hollow Fern/Brasswing.gp5")
            tabfinder.musicianOpensTabFinder()
            tabfinder.musicianPicksTheLibraryFolder()
            tabfinder.musicianTypesInTheArtistField("corv")

            tabfinder.musicianPicksTheSuggestion("Corvid Lane")

            tabfinder.assertArtistFieldShows("Corvid Lane")
            tabfinder.assertNoSuggestionShown("Corvid Lane")
            tabfinder.assertShown("Wexmoor", "1 / 2")
        }

    @Test
    fun theCrossEmptiesTheArtistFieldAndListsAllSongsAgain() =
        runTabFinderTest { tabfinder ->
            tabfinder.libraryFolderContains("Corvid Lane/Wexmoor.gp5", "Hollow Fern/Brasswing.gp5")
            tabfinder.musicianOpensTabFinder()
            tabfinder.musicianPicksTheLibraryFolder()
            tabfinder.musicianTypesInTheArtistField("corv")
            tabfinder.assertNotShown("Brasswing")

            tabfinder.musicianClearsTheArtistField()

            tabfinder.assertShown("Wexmoor", "Brasswing", "2 / 2")
        }

    @Test
    fun songsInHiddenFoldersAreSkipped() =
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

            tabfinder.assertStatus("No Songs found")
        }

    @Test
    fun aFolderWithoutSongsShowsNoSongsFound() =
        runTabFinderTest { tabfinder ->
            tabfinder.libraryFolderContains("notes.txt")
            tabfinder.musicianOpensTabFinder()

            tabfinder.musicianPicksTheLibraryFolder()

            tabfinder.assertStatus("No Songs found")
        }

    @Test
    fun aFolderTabFinderMayNotReadSaysWhy() =
        runTabFinderTest { tabfinder ->
            tabfinder.musicianOpensTabFinder()

            tabfinder.musicianPicksAFolderTabFinderMayNotRead()

            tabfinder.assertStatus("No folder chosen")
            tabfinder.assertShown("TabFinder may not read the Library folder.")
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
    fun reopeningTabFinderListsTheLastScanWithoutScanning() =
        runTabFinderTest { tabfinder ->
            tabfinder.libraryFolderContains("Wexmoor.gp5")
            tabfinder.musicianOpensTabFinder()
            tabfinder.musicianPicksTheLibraryFolder()
            tabfinder.assertStatus("1 Song")
            tabfinder.libraryFolderIsSlowToRead()

            tabfinder.musicianReopensTabFinder()

            tabfinder.assertStatus("1 Song")
            tabfinder.assertShown("Wexmoor")
        }

    @Test
    fun songsAddedSinceTheLastScanAreListedOnlyAfterARescan() =
        runTabFinderTest { tabfinder ->
            tabfinder.libraryFolderContains("Wexmoor.gp5")
            tabfinder.musicianOpensTabFinder()
            tabfinder.musicianPicksTheLibraryFolder()
            tabfinder.assertStatus("1 Song")
            tabfinder.libraryFolderContains("Brasswing.gp5")
            tabfinder.musicianReopensTabFinder()
            tabfinder.assertStatus("1 Song")

            tabfinder.musicianClicksRescan()

            tabfinder.assertStatus("2 Songs")
            tabfinder.assertShown("Brasswing")
        }

    @Test
    fun reopeningTabFinderWithALastScanItCannotReadScansTheLibraryAgain() =
        runTabFinderTest { tabfinder ->
            tabfinder.libraryFolderContains("Wexmoor.gp5")
            tabfinder.musicianOpensTabFinder()
            tabfinder.musicianPicksTheLibraryFolder()
            tabfinder.assertStatus("1 Song")
            tabfinder.libraryFolderContains("Brasswing.gp5")
            tabfinder.lastScanIsDamaged()

            tabfinder.musicianReopensTabFinder()

            tabfinder.assertStatus("2 Songs")
        }

    @Test
    fun rescanAfterTheLibraryFolderIsGoneShowsNoFolderChosen() =
        runTabFinderTest { tabfinder ->
            tabfinder.libraryFolderContains("Wexmoor.gp5")
            tabfinder.musicianOpensTabFinder()
            tabfinder.musicianPicksTheLibraryFolder()
            tabfinder.assertStatus("1 Song")
            tabfinder.libraryFolderIsDeleted()
            tabfinder.musicianReopensTabFinder()
            tabfinder.assertStatus("1 Song")

            tabfinder.musicianClicksRescan()

            tabfinder.assertStatus("No folder chosen")
        }
}
