package io.github.sbradl.tabfinder.desktop

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.ui.window.Window
import androidx.compose.ui.window.application
import io.github.sbradl.tabfinder.ui.TabFinder
import io.github.sbradl.tabfinder.ui.TabFinderCore

fun main() {
    val core = TabFinderCore()
    core.start()
    application {
        Window(onCloseRequest = ::exitApplication, title = "TabFinder") {
            MaterialTheme {
                Surface { TabFinder(core) }
            }
        }
    }
}
