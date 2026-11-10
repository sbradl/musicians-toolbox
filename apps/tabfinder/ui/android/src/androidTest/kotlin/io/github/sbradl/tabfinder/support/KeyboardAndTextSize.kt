package io.github.sbradl.tabfinder.support

import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.nativeKeyCode
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.semantics.SemanticsNode
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.semantics.getOrNull
import androidx.compose.ui.test.ComposeUiTest
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.SemanticsMatcher
import androidx.compose.ui.test.hasClickAction
import androidx.compose.ui.test.isFocused
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.unit.Density
import androidx.test.platform.app.InstrumentationRegistry
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue
import kotlin.test.fail

@Composable
fun WithTextScale(
    textScale: Float,
    content: @Composable () -> Unit,
) {
    val screen = LocalDensity.current
    CompositionLocalProvider(LocalDensity provides Density(screen.density, textScale), content = content)
}

@OptIn(ExperimentalTestApi::class)
fun ComposeUiTest.musicianPresses(key: Key) {
    InstrumentationRegistry.getInstrumentation().sendKeyDownUpSync(key.nativeKeyCode)
    waitForIdle()
}

@OptIn(ExperimentalTestApi::class)
fun ComposeUiTest.musicianTabsTo(label: String) {
    repeat(controls().size + 1) {
        musicianPresses(Key.Tab)

        if (focusedControlLabel() == label) {
            return
        }
    }
    fail("Tab never reaches \"$label\"")
}

@OptIn(ExperimentalTestApi::class)
fun ComposeUiTest.assertTabReachesEveryControl() {
    val controls = controls().map { control -> control.id }.toSet()
    val reached = mutableSetOf<Int>()
    repeat(controls.size + 1) {
        musicianPresses(Key.Tab)
        reached += onAllNodes(isFocused()).fetchSemanticsNodes().map { control -> control.id }
    }

    assertEquals(controls, reached intersect controls, "controls Tab never reaches")
}

@OptIn(ExperimentalTestApi::class)
fun ComposeUiTest.assertNoTextIsCutOffOrOverlapping() {
    val window = onRoot().fetchSemanticsNode().boundsInRoot
    val texts = onAllNodes(isText(), useUnmergedTree = true).fetchSemanticsNodes()
    texts.forEach { text -> assertFalse(text.isCutOffIn(window), "cut off: ${text.labels()}") }
    texts.forEach { text ->
        val overlapping = texts.filter { other -> other.id != text.id && other.boundsInRoot.overlapsStrictly(text.boundsInRoot) }

        assertEquals(emptyList(), overlapping.map { other -> other.labels() }, "overlapping ${text.labels()}")
    }
}

@OptIn(ExperimentalTestApi::class)
fun ComposeUiTest.assertEveryControlIsLabelled() {
    val controls = controls()
    assertTrue(controls.isNotEmpty())

    controls.forEach { control -> assertTrue(control.labels().any { it.isNotBlank() }, "unlabelled control: ${control.config}") }
}

@OptIn(ExperimentalTestApi::class)
private fun ComposeUiTest.controls() = onAllNodes(hasClickAction()).fetchSemanticsNodes()

@OptIn(ExperimentalTestApi::class)
private fun ComposeUiTest.focusedControlLabel(): String? {
    val focused = onAllNodes(isFocused()).fetchSemanticsNodes().firstOrNull() ?: return null

    return focused.labels().firstOrNull()
}

private fun isText() = SemanticsMatcher.keyIsDefined(SemanticsProperties.Text)

private fun SemanticsNode.isCutOffIn(window: Rect) = boundsInRoot.intersect(window) != boundsInRoot

private fun Rect.overlapsStrictly(other: Rect): Boolean {
    val overlap = intersect(other)

    return overlap.width > 0f && overlap.height > 0f
}

internal fun SemanticsNode.labels(): List<String> {
    val texts = config.getOrNull(SemanticsProperties.Text).orEmpty().map { it.text }
    val descriptions = config.getOrNull(SemanticsProperties.ContentDescription).orEmpty()

    return texts + descriptions
}
