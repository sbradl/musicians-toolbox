package io.github.sbradl.tabfinder.desktop

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.ui.window.Window
import androidx.compose.ui.window.application
import io.github.sbradl.tabfinder.core.desktopConfigDir
import io.github.sbradl.tabfinder.ui.TabFinder
import io.github.sbradl.tabfinder.ui.TabFinderCore

fun main() {
    val configDir = requireNotNull(desktopConfigDir()) { "Neither XDG_CONFIG_HOME nor HOME is set" }
    val core = TabFinderCore(PortalFolderPicker(), configDir)
    core.start()
    application {
        Window(onCloseRequest = ::exitApplication, title = "TabFinder") {
            MaterialTheme {
                Surface { TabFinder(core) }
            }
        }
    }
}
