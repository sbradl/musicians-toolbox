package io.github.sbradl.tabfinder

import androidx.compose.ui.test.ComposeUiTest
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.runComposeUiTest
import io.github.sbradl.tabfinder.ui.FolderPicker
import io.github.sbradl.tabfinder.ui.TabFinder
import io.github.sbradl.tabfinder.ui.TabFinderCore
import kotlin.io.path.createTempDirectory
import kotlin.io.path.deleteRecursively
import kotlin.test.AfterTest
import kotlin.test.Test

@OptIn(ExperimentalTestApi::class, kotlin.io.path.ExperimentalPathApi::class)
class LibraryScreenOnAndroidTest {
    private val configDir = createTempDirectory("tabfinder-config")

    @AfterTest
    fun deleteConfigDir() {
        configDir.deleteRecursively()
    }

    @Test
    fun noFolderChosenOffersChooseFolder() =
        runComposeUiTest {
            startTabFinder()

            onNodeWithText("No folder chosen").assertExists()
            onNodeWithText("Choose folder").assertExists()
        }

    private fun ComposeUiTest.startTabFinder() {
        val core = TabFinderCore(FolderPicker { null }, configDir.toString())
        core.start()
        setContent { TabFinder(core) }
    }
}
