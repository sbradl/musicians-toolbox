package io.github.sbradl.tabfinder.desktop

import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.DpSize
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Window
import androidx.compose.ui.window.application
import androidx.compose.ui.window.rememberWindowState
import io.github.sbradl.tabfinder.core.desktopConfigDir
import io.github.sbradl.tabfinder.ui.TabFinder
import io.github.sbradl.tabfinder.ui.TabFinderCore
import io.github.sbradl.tabfinder.ui.TabFinderTheme
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.runBlocking

private const val APP_NAME = "TabFinder"
private val DEFAULT_WIDTH = 800.dp
private val DEFAULT_HEIGHT = 600.dp

fun main() {
    val core = startCore()
    showWindow(core)
}

private fun startCore(): TabFinderCore {
    val configDir = requireNotNull(desktopConfigDir()) { "Neither XDG_CONFIG_HOME nor HOME is set" }
    val core = TabFinderCore.onDesktop(PortalFolderPicker(), configDir)
    core.start()

    return core
}

private fun showWindow(core: TabFinderCore) {
    val desktopScale = DesktopScale.ofThisDesktop()
    val desktopPrefersDark = runBlocking { PortalAppearance().followPrefersDark(CoroutineScope(Dispatchers.IO)) }
    application {
        val prefersDark by desktopPrefersDark.collectAsState()
        val windowState = rememberWindowState(size = desktopScale.applyTo(DpSize(DEFAULT_WIDTH, DEFAULT_HEIGHT)))
        Window(onCloseRequest = ::exitApplication, title = APP_NAME, state = windowState) {
            CompositionLocalProvider(LocalDensity provides desktopScale.applyTo(LocalDensity.current)) {
                TabFinderTheme(prefersDark) { TabFinder(core, APP_NAME) }
            }
        }
    }
}
