package io.github.sbradl.tabfinder.ui

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
import kotlin.test.Test
import kotlin.test.assertTrue

@OptIn(ExperimentalTestApi::class, kotlin.io.path.ExperimentalPathApi::class)
class LibraryScreenTest {
    private fun withLibraryFolder(
        vararg files: String,
        test: (Path) -> Unit,
    ) {
        val folder = createTempDirectory("tabfinder")
        try {
            files.forEach { file ->
                folder.resolve(file).also { it.parent.createDirectories() }.createFile()
            }
            test(folder)
        } finally {
            folder.deleteRecursively()
        }
    }

    private fun ComposeUiTest.showTabFinder(pickedFolder: Path?) {
        val core = TabFinderCore(FolderPicker { pickedFolder?.toString() })
        core.start()
        setContent { TabFinder(core) }
    }

    @Test
    fun noFolderChosenOffersChooseFolder() =
        runComposeUiTest {
            showTabFinder(pickedFolder = null)

            onNodeWithText("No folder chosen").assertExists()
            onNodeWithText("Choose folder").assertExists()
        }

    @Test
    fun choosingAFolderWithTabFilesShowsItsSongs() =
        withLibraryFolder("Wonderwall.gp5", "Beatles/Blackbird.gp5", "notes.txt") { folder ->
            runComposeUiTest {
                showTabFinder(pickedFolder = folder)

                onNodeWithText("Choose folder").performClick()

                waitUntilExactlyOneExists(hasText("2 Songs"))
                onNodeWithText("Blackbird").assertExists()
                onNodeWithText("Wonderwall").assertExists()
            }
        }

    @Test
    fun choosingAFolderWithoutTabFilesShowsNoTabFilesFound() =
        withLibraryFolder("notes.txt") { folder ->
            runComposeUiTest {
                showTabFinder(pickedFolder = folder)

                onNodeWithText("Choose folder").performClick()

                waitUntilExactlyOneExists(hasText("No Tab files found"))
            }
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
        withLibraryFolder("Wonderwall.gp5") { folder ->
            runComposeUiTest {
                showTabFinder(pickedFolder = folder)
                onNodeWithText("Choose folder").performClick()
                waitUntilExactlyOneExists(hasText("1 Song"))

                val interactiveElements = onAllNodes(hasClickAction()).fetchSemanticsNodes()
                assertTrue(interactiveElements.isNotEmpty())
                interactiveElements.forEach { element ->
                    val labels =
                        element.config
                            .getOrNull(SemanticsProperties.Text)
                            .orEmpty()
                            .map { it.text } +
                            element.config.getOrNull(SemanticsProperties.ContentDescription).orEmpty()
                    assertTrue(labels.any { it.isNotBlank() }, "unlabelled element: $element")
                }
            }
        }
}
