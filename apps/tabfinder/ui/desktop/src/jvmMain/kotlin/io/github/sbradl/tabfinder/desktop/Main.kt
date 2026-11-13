package io.github.sbradl.tabfinder.desktop

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.window.Window
import androidx.compose.ui.window.application
import io.github.sbradl.tabfinder.core.desktopConfigDir
import io.github.sbradl.tabfinder.ui.TabFinder
import io.github.sbradl.tabfinder.ui.TabFinderCore
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.runBlocking

private const val APP_NAME = "TabFinder"

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
    val desktopPrefersDark = runBlocking { PortalAppearance().followPrefersDark(CoroutineScope(Dispatchers.IO)) }
    application {
        val prefersDark by desktopPrefersDark.collectAsState()
        Window(onCloseRequest = ::exitApplication, title = APP_NAME) {
            MaterialTheme(colorScheme = if (prefersDark) darkColorScheme() else lightColorScheme()) {
                Surface { TabFinder(core, APP_NAME) }
            }
        }
    }
}
