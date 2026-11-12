package io.github.sbradl.tabfinder.ui.support

import androidx.compose.ui.input.key.Key
import androidx.compose.ui.test.ComposeUiTest
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assertIsNotEnabled
import androidx.compose.ui.test.hasClickAction
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.runComposeUiTest
import androidx.compose.ui.test.waitUntilExactlyOneExists
import io.github.sbradl.tabfinder.ui.FolderPick
import io.github.sbradl.tabfinder.ui.FolderPicker
import io.github.sbradl.tabfinder.ui.TabFinder
import io.github.sbradl.tabfinder.ui.TabFinderCore
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
    private var pickerAnswer: FolderPick = FolderPick.Cancelled
    private var slowLibraryFolder: LibraryFolderThatCanBeHeld? = null
    private var textScale = 2f

    fun libraryFolderContains(vararg files: String) {
        files.forEach { file ->
            val path = libraryFolder.resolve(file)
            path.parent.createDirectories()
            path.createFile()
        }
    }

    fun libraryFolderIsSlowToRead() {
        val folder = LibraryFolderThatCanBeHeld(LibraryFolderOnDisk(libraryFolder))
        folder.holdReading()
        slowLibraryFolder = folder
    }

    fun libraryFolderIsDeleted() {
        libraryFolder.deleteRecursively()
    }

    fun musicianDoublesTheTextSize() {
        textScale = 2f
    }

    fun musicianOpensTabFinder() {
        val core = startTabFinderCore()
        core.start()
        ui.waitUntil { core.viewModel.value != null }
        ui.setContent { WithTextScale(textScale) { TabFinder(core, appName = "TabFinder") } }
    }

    fun musicianPicksTheLibraryFolder() {
        pickerAnswer = FolderPick.Picked(libraryFolder.toUri().toString())
        clickChooseFolder()
    }

    fun musicianPicksTheLibraryFolderWithTheKeyboard(confirmKey: Key) {
        pickerAnswer = FolderPick.Picked(libraryFolder.toUri().toString())
        ui.musicianTabsTo("Choose folder")
        ui.musicianPresses(confirmKey)
    }

    fun musicianFindsNoFolderDialog() {
        pickerAnswer = FolderPick.Unavailable
        clickChooseFolder()
    }

    fun musicianClicksRescan() {
        ui.onNodeWithText("Rescan").performClick()
    }

    fun assertMessageShown(message: String) {
        ui.waitUntilExactlyOneExists(hasText(message))
    }

    fun assertStatus(status: String) {
        ui.waitUntilExactlyOneExists(hasText(status))
    }

    fun assertChooseFolderIsDisabled() {
        ui.onNodeWithText("Choose folder").assertIsNotEnabled()
    }

    fun assertShown(vararg texts: String) {
        texts.forEach { text -> ui.onNodeWithText(text).assertExists() }
    }

    fun assertTabReachesEveryControl() = ui.assertTabReachesEveryControl()

    fun assertNoTextIsCutOffOrOverlapping() = ui.assertNoTextIsCutOffOrOverlapping()

    fun assertEveryInteractiveElementIsLabelled() {
        val interactiveElements = ui.onAllNodes(hasClickAction()).fetchSemanticsNodes()
        assertTrue(interactiveElements.isNotEmpty())
        interactiveElements.forEach { element ->
            assertTrue(element.labels().any { it.isNotBlank() }, "unlabelled element: $element")
        }
    }

    private fun clickChooseFolder() {
        ui.onNodeWithText("Choose folder").performClick()
    }

    private fun startTabFinderCore(): TabFinderCore {
        val folderPicker = FolderPicker { _ -> pickerAnswer }
        val slowFolder = slowLibraryFolder

        if (slowFolder != null) {
            return TabFinderCore.withLibraryFolder(folderPicker, configDir.toString(), slowFolder)
        }

        return TabFinderCore.onDesktop(folderPicker, configDir.toString())
    }

    internal fun deleteFolders() {
        slowLibraryFolder?.releaseReading()
        libraryFolder.deleteRecursively()
        configDir.deleteRecursively()
    }
}
