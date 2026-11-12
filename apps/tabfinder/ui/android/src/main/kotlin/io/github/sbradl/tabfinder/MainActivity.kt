package io.github.sbradl.tabfinder

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.lifecycle.ViewModelProvider
import io.github.sbradl.tabfinder.ui.TabFinder

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val tabfinder = ViewModelProvider(this)[RunningTabFinder::class.java]
        tabfinder.folderPicker.attachTo(activityResultRegistry)
        setContent {
            MaterialTheme {
                Surface { TabFinder(tabfinder.core, appName()) }
            }
        }
    }

    private fun appName() = applicationInfo.loadLabel(packageManager).toString()
}
