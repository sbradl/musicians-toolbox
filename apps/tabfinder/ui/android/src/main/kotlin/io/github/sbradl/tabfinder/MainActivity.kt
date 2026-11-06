package io.github.sbradl.tabfinder

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import io.github.sbradl.tabfinder.ui.TabFinder
import io.github.sbradl.tabfinder.ui.TabFinderCore

class MainActivity : ComponentActivity() {
    private val folderPicker = SafFolderPicker(this)

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val core = startCore()
        setContent {
            MaterialTheme {
                Surface { TabFinder(core) }
            }
        }
    }

    private fun startCore(): TabFinderCore {
        val core = TabFinderCore.onAndroid(folderPicker, filesDir.path, SafLibraryFolder(contentResolver))
        core.start()

        return core
    }
}
