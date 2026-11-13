package io.github.sbradl.tabfinder.ui

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.runComposeUiTest
import kotlin.test.Test
import kotlin.test.assertEquals

@OptIn(ExperimentalTestApi::class)
class DarkAndLightTest {
    @Test
    fun aSystemThatPrefersDarkShowsTabFinderDark() {
        val background = backgroundWhenTheSystemPrefersDark(true)

        assertEquals(darkColorScheme().background, background)
    }

    @Test
    fun aSystemThatPrefersLightShowsTabFinderLight() {
        val background = backgroundWhenTheSystemPrefersDark(false)

        assertEquals(lightColorScheme().background, background)
    }

    private fun backgroundWhenTheSystemPrefersDark(prefersDark: Boolean): Color {
        var background = Color.Unspecified
        runComposeUiTest {
            setContent {
                TabFinderTheme(prefersDark) { background = MaterialTheme.colorScheme.background }
            }
            waitForIdle()
        }

        return background
    }
}
