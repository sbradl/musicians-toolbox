package io.github.sbradl.tabfinder

import android.provider.DocumentsContract
import androidx.compose.ui.test.ComposeUiTest
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.runComposeUiTest
import androidx.compose.ui.test.waitUntilExactlyOneExists
import androidx.test.platform.app.InstrumentationRegistry
import io.github.sbradl.tabfinder.ui.FolderPicker
import io.github.sbradl.tabfinder.ui.TabFinder
import io.github.sbradl.tabfinder.ui.TabFinderCore
import java.io.File
import kotlin.io.path.createTempDirectory
import kotlin.io.path.deleteRecursively
import kotlin.test.AfterTest
import kotlin.test.Test

@OptIn(ExperimentalTestApi::class, kotlin.io.path.ExperimentalPathApi::class)
class LibraryScreenOnAndroidTest {
    private val app = InstrumentationRegistry.getInstrumentation().targetContext
    private val configDir = createTempDirectory("tabfinder-config")
    private val libraryFolder = File(app.cacheDir, "library")

    @AfterTest
    fun deleteFolders() {
        configDir.deleteRecursively()
        libraryFolder.deleteRecursively()
    }

    @Test
    fun noFolderChosenOffersChooseFolder() =
        runComposeUiTest {
            startTabFinder(pickedFolder = null)

            onNodeWithText("No folder chosen").assertExists()
            onNodeWithText("Choose folder").assertExists()
        }

    @Test
    fun tabFilesInSubfoldersAreFoundIgnoringTheCaseOfTheirExtension() =
        runComposeUiTest {
            libraryFolderContains("Wonderwall.GP5", "Beatles/Blackbird.tg", "notes.txt")
            startTabFinder(pickedFolder = libraryFolderUri())

            clickChooseFolder()

            waitUntilShown("2 Songs")
            onNodeWithText("Blackbird").assertExists()
            onNodeWithText("Wonderwall").assertExists()
        }

    @Test
    fun tabFilesInHiddenFoldersAreSkipped() =
        runComposeUiTest {
            libraryFolderContains("Wonderwall.gp5", ".backup/Old Wonderwall.gp5")
            startTabFinder(pickedFolder = libraryFolderUri())

            clickChooseFolder()

            waitUntilShown("1 Song")
            onNodeWithText("Old Wonderwall").assertDoesNotExist()
        }

    @Test
    fun aFileNamedOnlyByAnExtensionIsNoSong() =
        runComposeUiTest {
            libraryFolderContains(".gp5")
            startTabFinder(pickedFolder = libraryFolderUri())

            clickChooseFolder()

            waitUntilShown("No Tab files found")
        }

    @Test
    fun aFolderWithoutTabFilesShowsNoTabFilesFound() =
        runComposeUiTest {
            libraryFolderContains("notes.txt")
            startTabFinder(pickedFolder = libraryFolderUri())

            clickChooseFolder()

            waitUntilShown("No Tab files found")
        }

    private fun libraryFolderContains(vararg files: String) {
        files.forEach { file ->
            val path = File(libraryFolder, file)
            path.parentFile?.mkdirs()
            path.createNewFile()
        }
    }

    private fun libraryFolderUri() = DocumentsContract.buildTreeDocumentUri(LibraryFolderForTests.AUTHORITY, libraryFolder.name).toString()

    private fun ComposeUiTest.startTabFinder(pickedFolder: String?) {
        val core = TabFinderCore(FolderPicker { pickedFolder }, configDir.toString(), SafLibraryFolder(app.contentResolver))
        core.start()
        setContent { TabFinder(core) }
    }

    private fun ComposeUiTest.clickChooseFolder() {
        onNodeWithText("Choose folder").performClick()
    }

    private fun ComposeUiTest.waitUntilShown(text: String) {
        waitUntilExactlyOneExists(hasText(text))
    }
}
