package io.github.sbradl.tabfinder.support

import android.provider.DocumentsContract
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
    private var folderThePickerAnswers: String? = null

    fun libraryFolderContains(vararg files: String) {
        files.forEach { file ->
            val path = File(libraryFolder, file)
            path.parentFile?.mkdirs()
            path.createNewFile()
        }
    }

    fun musicianOpensTabFinder() {
        val folderPicker = FolderPicker { folderThePickerAnswers }
        val core = TabFinderCore.onAndroid(folderPicker, configDir.path, SafLibraryFolder(app.contentResolver))
        core.start()
        ui.setContent { TabFinder(core) }
    }

    fun musicianPicksTheLibraryFolder() {
        folderThePickerAnswers = libraryFolderTree()
        ui.onNodeWithText("Choose folder").performClick()
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

    private fun libraryFolderTree() = DocumentsContract.buildTreeDocumentUri(LibraryFolderForTests.AUTHORITY, libraryFolder.name).toString()
}
