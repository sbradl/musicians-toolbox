package io.github.sbradl.tabfinder.desktop

import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.DpSize
import androidx.compose.ui.unit.dp
import kotlin.test.Test
import kotlin.test.assertEquals

class DesktopScaleTest {
    @Test
    fun aDesktopScaledTo170PercentDrawsTabFinder170PercentLarger() {
        val desktop = desktopPublishing(windowScalingFactor = 1, unscaledDpi = 163.2f)

        val scaled = DesktopScale(desktop, javaScale = 1f).applyTo(Density(1f))

        assertDensity(1.7f, scaled)
    }

    @Test
    fun aDesktopScaledTo200PercentByJavaItselfIsNotScaledTwice() {
        val desktop = desktopPublishing(windowScalingFactor = 2, unscaledDpi = 96f)

        val scaled = DesktopScale(desktop, javaScale = 2f).applyTo(Density(2f))

        assertDensity(2f, scaled)
    }

    @Test
    fun javaScaledByHandToTheDesktopsScaleIsNotScaledTwice() {
        val desktop = desktopPublishing(windowScalingFactor = 1, unscaledDpi = 192f)

        val scaled = DesktopScale(desktop, javaScale = 2f).applyTo(Density(2f))

        assertDensity(2f, scaled)
    }

    @Test
    fun javaScaledByHandBeyondTheDesktopsScaleStaysAsItIs() {
        val desktop = desktopPublishing(windowScalingFactor = 1, unscaledDpi = 163.2f)

        val scaled = DesktopScale(desktop, javaScale = 2f).applyTo(Density(2f))

        assertDensity(2f, scaled)
    }

    @Test
    fun aDesktopPublishingNoDpiLeavesTheScaleAsItIs() {
        val scaled = DesktopScale({ null }, javaScale = 1f).applyTo(Density(1f))

        assertDensity(1f, scaled)
    }

    @Test
    fun aDesktopPublishingLessThanTheUsualDpiLeavesTheScaleAsItIs() {
        val desktop = desktopPublishing(windowScalingFactor = 1, unscaledDpi = 72f)

        val scaled = DesktopScale(desktop, javaScale = 1f).applyTo(Density(1f))

        assertDensity(1f, scaled)
    }

    @Test
    fun theWindowOpensAsMuchLargerAsTheDesktopScale() {
        val desktop = desktopPublishing(windowScalingFactor = 1, unscaledDpi = 163.2f)

        val size = DesktopScale(desktop, javaScale = 1f).applyTo(DpSize(800.dp, 600.dp))

        assertSize(DpSize(1360.dp, 1020.dp), size)
    }

    @Test
    fun theTextSizeTheMusicianChoseStays() {
        val desktop = desktopPublishing(windowScalingFactor = 1, unscaledDpi = 163.2f)

        val scaled = DesktopScale(desktop, javaScale = 1f).applyTo(Density(1f, fontScale = 1.5f))

        assertEquals(1.5f, scaled.fontScale)
    }

    private fun desktopPublishing(
        windowScalingFactor: Int,
        unscaledDpi: Float,
    ): (String) -> Any? =
        { setting ->
            when (setting) {
                "gnome.Gdk/WindowScalingFactor" -> windowScalingFactor
                "gnome.Gdk/UnscaledDPI" -> (unscaledDpi * 1024).toInt()
                else -> null
            }
        }

    private fun assertDensity(
        expected: Float,
        actual: Density,
    ) = assertEquals(expected, actual.density, absoluteTolerance = 0.01f)

    private fun assertSize(
        expected: DpSize,
        actual: DpSize,
    ) {
        assertEquals(expected.width.value, actual.width.value, absoluteTolerance = 1f)
        assertEquals(expected.height.value, actual.height.value, absoluteTolerance = 1f)
    }
}
