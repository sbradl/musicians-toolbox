package io.github.sbradl.tabfinder.support

import android.provider.DocumentsContract
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.test.ComposeUiTest
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.hasSetTextAction
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.test.runComposeUiTest
import androidx.compose.ui.test.waitUntilAtLeastOneExists
import androidx.compose.ui.test.waitUntilDoesNotExist
import androidx.compose.ui.test.waitUntilExactlyOneExists
import androidx.test.platform.app.InstrumentationRegistry
import io.github.sbradl.tabfinder.LibraryFolderForTests
import io.github.sbradl.tabfinder.SafLibraryFolder
import io.github.sbradl.tabfinder.ui.FolderPick
import io.github.sbradl.tabfinder.ui.FolderPicker
import io.github.sbradl.tabfinder.ui.TabFinder
import io.github.sbradl.tabfinder.ui.TabFinderCore
import java.io.File

@OptIn(ExperimentalTestApi::class)
fun runTabFinderTest(test: (TabFinderOnAndroid) -> Unit) =
    runComposeUiTest {
        val tabfinder = TabFinderOnAndroid(this)
        try {
            test(tabfinder)
        } finally {
            tabfinder.deleteFolders()
        }
    }

@OptIn(ExperimentalTestApi::class)
class TabFinderOnAndroid(
    private val ui: ComposeUiTest,
) {
    private val app = InstrumentationRegistry.getInstrumentation().targetContext
    private val appName = app.applicationInfo.loadLabel(app.packageManager).toString()
    private val configDir = File(app.cacheDir, "config")
    private val libraryFolder = File(app.cacheDir, "library")
    private var pickerAnswer: FolderPick = FolderPick.Cancelled
    private val libraryFolderReader = LibraryFolderThatCanBeHeld(SafLibraryFolder(app.contentResolver))
    private var runningTabFinder by mutableStateOf<TabFinderCore?>(null)
    private var textScale = 1f

    fun libraryFolderContains(vararg files: String) {
        files.forEach { file ->
            val path = File(libraryFolder, file)
            path.parentFile?.mkdirs()
            path.createNewFile()
        }
    }

    fun libraryFolderIsSlowToRead() {
        libraryFolderReader.holdReading()
    }

    fun libraryFolderFinishesReading() {
        libraryFolderReader.releaseReading()
    }

    fun libraryFolderIsDeleted() {
        libraryFolder.deleteRecursively()
    }

    fun musicianDoublesTheTextSize() {
        textScale = 2f
    }

    fun musicianOpensTabFinder() {
        startTabFinder()
        ui.setContent {
            val core = runningTabFinder

            if (core != null) {
                WithTextScale(textScale) { TabFinder(core, appName) }
            }
        }
    }

    fun musicianReopensTabFinder() {
        closeTabFinder()
        startTabFinder()
    }

    fun musicianPicksTheLibraryFolder() {
        pickerAnswer = FolderPick.Picked(treeOf(libraryFolder.name))
        clickChooseFolder()
    }

    fun musicianPicksTheLibraryFolderWithTheKeyboard() {
        pickerAnswer = FolderPick.Picked(treeOf(libraryFolder.name))
        ui.musicianTabsTo("Choose folder")
        ui.musicianPresses(Key.Enter)
    }

    fun musicianPicksAFolderTabFinderMayNotRead() {
        pickerAnswer = FolderPick.Picked(treeOf(LibraryFolderForTests.FOLDER_IT_MAY_NOT_READ))
        clickChooseFolder()
    }

    fun musicianClicksRescan() {
        ui.onNodeWithText("Rescan").performClick()
    }

    fun musicianTypesInTheSongField(text: String) {
        ui.onNode(hasSetTextAction() and hasText("Song")).performTextInput(text)
    }

    fun musicianClearsTheSongField() {
        ui.onNodeWithContentDescription("Clear Song").performClick()
    }

    fun musicianClicksClearFilters() {
        ui.onNodeWithText("Clear filters").performClick()
    }

    fun assertStatus(status: String) {
        ui.waitUntilExactlyOneExists(hasText(status))
    }

    fun assertShown(vararg texts: String) {
        texts.forEach { text -> ui.waitUntilAtLeastOneExists(hasText(text)) }
    }

    fun assertTabReachesEveryControl() = ui.assertTabReachesEveryControl()

    fun assertNoTextIsCutOffOrOverlapping() = ui.assertNoTextIsCutOffOrOverlapping()

    fun assertEveryControlIsLabelled() = ui.assertEveryControlIsLabelled()

    fun assertNotShown(text: String) {
        ui.waitUntilDoesNotExist(hasText(text))
    }

    internal fun deleteFolders() {
        configDir.deleteRecursively()
        libraryFolder.deleteRecursively()
    }

    private fun clickChooseFolder() {
        ui.onNodeWithText("Choose folder").performClick()
    }

    private fun startTabFinder() {
        val folderPicker = FolderPicker { _ -> pickerAnswer }
        val core = TabFinderCore.withLibraryFolder(folderPicker, configDir.path, libraryFolderReader)
        core.start()
        runningTabFinder = core
        ui.waitUntil { core.viewModel.value != null }
    }

    private fun closeTabFinder() {
        runningTabFinder = null
        ui.waitForIdle()
    }

    private fun treeOf(folder: String) = DocumentsContract.buildTreeDocumentUri(LibraryFolderForTests.AUTHORITY, folder).toString()
}
