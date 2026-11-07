package io.github.sbradl.tabfinder.support

import android.provider.DocumentsContract
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.test.ComposeUiTest
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.runComposeUiTest
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
    private val configDir = File(app.cacheDir, "config")
    private val libraryFolder = File(app.cacheDir, "library")
    private var pickerAnswer: FolderPick = FolderPick.Cancelled
    private val libraryFolderReader = LibraryFolderThatCanBeHeld(SafLibraryFolder(app.contentResolver))
    private var runningTabFinder by mutableStateOf<TabFinderCore?>(null)

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

    fun musicianOpensTabFinder() {
        startTabFinder()
        ui.setContent {
            val core = runningTabFinder

            if (core != null) {
                TabFinder(core)
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

    fun musicianPicksAFolderTabFinderMayNotRead() {
        pickerAnswer = FolderPick.Picked(treeOf(LibraryFolderForTests.FOLDER_IT_MAY_NOT_READ))
        clickChooseFolder()
    }

    fun assertStatus(status: String) {
        ui.waitUntilExactlyOneExists(hasText(status))
    }

    fun assertShown(vararg texts: String) {
        texts.forEach { text -> ui.onNodeWithText(text).assertExists() }
    }

    fun assertNotShown(text: String) {
        ui.onNodeWithText(text).assertDoesNotExist()
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
