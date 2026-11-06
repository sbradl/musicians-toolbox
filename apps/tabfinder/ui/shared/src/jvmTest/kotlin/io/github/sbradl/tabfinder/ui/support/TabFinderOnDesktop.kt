package io.github.sbradl.tabfinder.ui.support

import androidx.compose.ui.semantics.SemanticsNode
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
import androidx.compose.ui.test.ComposeUiTest
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.hasClickAction
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.runComposeUiTest
import androidx.compose.ui.test.waitUntilExactlyOneExists
import io.github.sbradl.tabfinder.ui.FolderPicker
import io.github.sbradl.tabfinder.ui.TabFinder
import io.github.sbradl.tabfinder.ui.TabFinderCore
import java.nio.file.Path
import kotlin.io.path.ExperimentalPathApi
import kotlin.io.path.createDirectories
import kotlin.io.path.createFile
import kotlin.io.path.createTempDirectory
import kotlin.io.path.deleteRecursively
import kotlin.test.assertTrue

@OptIn(ExperimentalTestApi::class)
fun runTabFinderTest(test: (TabFinderOnDesktop) -> Unit) =
    runComposeUiTest {
        val tabfinder = TabFinderOnDesktop(this)
        try {
            test(tabfinder)
        } finally {
            tabfinder.deleteFolders()
        }
    }

@OptIn(ExperimentalTestApi::class, ExperimentalPathApi::class)
class TabFinderOnDesktop(
    private val ui: ComposeUiTest,
) {
    private val libraryFolder = createTempDirectory("tabfinder")
    private val configDir = createTempDirectory("tabfinder-config")
    private var folderThePickerAnswers: Path? = null

    fun libraryFolderContains(vararg files: String) {
        files.forEach { file ->
            val path = libraryFolder.resolve(file)
            path.parent.createDirectories()
            path.createFile()
        }
    }

    fun musicianOpensTabFinder() {
        val core = TabFinderCore.onDesktop(FolderPicker { folderThePickerAnswers?.toString() }, configDir.toString())
        core.start()
        ui.setContent { TabFinder(core) }
    }

    fun musicianPicksTheLibraryFolder() {
        folderThePickerAnswers = libraryFolder
        ui.onNodeWithText("Choose folder").performClick()
    }

    fun assertStatus(status: String) {
        ui.waitUntilExactlyOneExists(hasText(status))
    }

    fun assertShown(vararg texts: String) {
        texts.forEach { text -> ui.onNodeWithText(text).assertExists() }
    }

    fun assertEveryInteractiveElementIsLabelled() {
        val interactiveElements = ui.onAllNodes(hasClickAction()).fetchSemanticsNodes()
        assertTrue(interactiveElements.isNotEmpty())
        interactiveElements.forEach { element ->
            assertTrue(element.labels().any { it.isNotBlank() }, "unlabelled element: $element")
        }
    }

    internal fun deleteFolders() {
        libraryFolder.deleteRecursively()
        configDir.deleteRecursively()
    }

    private fun SemanticsNode.labels(): List<String> {
        val texts = config.getOrNull(SemanticsProperties.Text).orEmpty().map { it.text }
        val descriptions = config.getOrNull(SemanticsProperties.ContentDescription).orEmpty()

        return texts + descriptions
    }
}
