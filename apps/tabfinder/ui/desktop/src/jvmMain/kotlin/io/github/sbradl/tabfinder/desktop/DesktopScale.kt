package io.github.sbradl.tabfinder.desktop

import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.DpSize
import java.awt.GraphicsEnvironment
import java.awt.Toolkit

class DesktopScale(
    desktopSetting: (String) -> Any?,
    javaScale: Float,
) {
    private val scaleJavaMisses = scaleJavaMisses(desktopSetting, javaScale)

    fun applyTo(density: Density) = Density(density.density * scaleJavaMisses, density.fontScale)

    fun applyTo(size: DpSize) = size * scaleJavaMisses

    companion object {
        private const val UNSCALED_DPI = "gnome.Gdk/UnscaledDPI"
        private const val WINDOW_SCALING_FACTOR = "gnome.Gdk/WindowScalingFactor"
        private const val XSETTINGS_DPI_UNIT = 1024
        private const val USUAL_DPI = 96f

        fun ofThisDesktop() = DesktopScale(Toolkit.getDefaultToolkit()::getDesktopProperty, javaScaleOfThisScreen())

        private fun javaScaleOfThisScreen(): Float {
            val screen = GraphicsEnvironment.getLocalGraphicsEnvironment().defaultScreenDevice

            return screen.defaultConfiguration.defaultTransform.scaleX
                .toFloat()
        }

        private fun scaleJavaMisses(
            desktopSetting: (String) -> Any?,
            javaScale: Float,
        ): Float {
            val unscaledDpi = desktopSetting(UNSCALED_DPI) as? Int ?: return 1f
            val windowScalingFactor = desktopSetting(WINDOW_SCALING_FACTOR) as? Int ?: 1
            val desktopScale = unscaledDpi.toFloat() / XSETTINGS_DPI_UNIT / USUAL_DPI * windowScalingFactor

            return (desktopScale / javaScale).coerceAtLeast(1f)
        }
    }
}
