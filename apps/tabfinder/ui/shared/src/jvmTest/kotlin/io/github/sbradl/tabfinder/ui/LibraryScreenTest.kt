package io.github.sbradl.tabfinder.ui

import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.hasClickAction
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.runComposeUiTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

@OptIn(ExperimentalTestApi::class)
class LibraryScreenTest {
    @Test
    fun noFolderChosenRendersAndChooseFolderReachesTheCore() =
        runComposeUiTest {
            val core = TabFinderCore()
            var chooseFolderCalls = 0
            core.start()

            setContent {
                TabFinder(core, onChooseFolder = {
                    chooseFolderCalls++
                    core.chooseFolder()
                })
            }

            onNodeWithText("No folder chosen").assertExists()
            onNodeWithText("Choose folder").performClick()
            assertEquals(1, chooseFolderCalls)
        }

    @Test
    fun everyInteractiveElementHasALabel() =
        runComposeUiTest {
            val core = TabFinderCore()
            core.start()

            setContent { TabFinder(core) }

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
