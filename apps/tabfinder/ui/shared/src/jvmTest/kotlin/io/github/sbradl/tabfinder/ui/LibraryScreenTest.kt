package io.github.sbradl.tabfinder.ui

import androidx.compose.ui.semantics.SemanticsNode
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
import androidx.compose.ui.test.ComposeUiTest
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assertIsNotEnabled
import androidx.compose.ui.test.hasClickAction
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.runComposeUiTest
import androidx.compose.ui.test.waitUntilExactlyOneExists
import io.github.sbradl.tabfinder.core.Action
import io.github.sbradl.tabfinder.core.LibraryViewModel
import java.nio.file.Path
import kotlin.io.path.createDirectories
import kotlin.io.path.createFile
import kotlin.io.path.createTempDirectory
import kotlin.io.path.deleteRecursively
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertTrue

@OptIn(ExperimentalTestApi::class, kotlin.io.path.ExperimentalPathApi::class)
class LibraryScreenTest {
    private val libraryFolder = createTempDirectory("tabfinder")
    private val configDir = createTempDirectory("tabfinder-config")

    @AfterTest
    fun deleteTempDirs() {
        libraryFolder.deleteRecursively()
        configDir.deleteRecursively()
    }

    @Test
    fun noFolderChosenOffersChooseFolder() =
        runComposeUiTest {
            startTabFinder(pickedFolder = null)

            onNodeWithText("No folder chosen").assertExists()
            onNodeWithText("Choose folder").assertExists()
        }

    @Test
    fun choosingAFolderWithTabFilesShowsItsSongs() =
        runComposeUiTest {
            libraryFolderContains("Wonderwall.gp5", "Beatles/Blackbird.gp5", "notes.txt")
            startTabFinder(pickedFolder = libraryFolder)

            clickChooseFolder()

            waitUntilShown("2 Songs")
            onNodeWithText("Blackbird").assertExists()
            onNodeWithText("Wonderwall").assertExists()
        }

    @Test
    fun choosingAFolderWithoutTabFilesShowsNoTabFilesFound() =
        runComposeUiTest {
            libraryFolderContains("notes.txt")
            startTabFinder(pickedFolder = libraryFolder)

            clickChooseFolder()

            waitUntilShown("No Tab files found")
        }

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
        runComposeUiTest {
            libraryFolderContains("Wonderwall.gp5")
            startTabFinder(pickedFolder = libraryFolder)
            clickChooseFolder()
            waitUntilShown("1 Song")

            assertEveryInteractiveElementIsLabelled()
        }

    private fun libraryFolderContains(vararg files: String) {
        files.forEach { file ->
            val path = libraryFolder.resolve(file)
            path.parent.createDirectories()
            path.createFile()
        }
    }

    private fun ComposeUiTest.startTabFinder(pickedFolder: Path?) {
        val core = TabFinderCore(FolderPicker { pickedFolder?.toString() }, configDir.toString())
        core.start()
        setContent { TabFinder(core) }
    }

    private fun ComposeUiTest.clickChooseFolder() {
        onNodeWithText("Choose folder").performClick()
    }

    private fun ComposeUiTest.waitUntilShown(text: String) {
        waitUntilExactlyOneExists(hasText(text))
    }

    private fun ComposeUiTest.assertEveryInteractiveElementIsLabelled() {
        val interactiveElements = onAllNodes(hasClickAction()).fetchSemanticsNodes()
        assertTrue(interactiveElements.isNotEmpty())
        interactiveElements.forEach { element ->
            assertTrue(element.labels().any { it.isNotBlank() }, "unlabelled element: $element")
        }
    }

    private fun SemanticsNode.labels(): List<String> {
        val texts = config.getOrNull(SemanticsProperties.Text).orEmpty().map { it.text }
        val descriptions = config.getOrNull(SemanticsProperties.ContentDescription).orEmpty()

        return texts + descriptions
    }
}
