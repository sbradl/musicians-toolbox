package io.github.sbradl.tabfinder

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.lifecycle.ViewModelProvider
import io.github.sbradl.tabfinder.ui.TabFinder
import io.github.sbradl.tabfinder.ui.TabFinderTheme

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val tabfinder = ViewModelProvider(this)[RunningTabFinder::class.java]
        tabfinder.folderPicker.attachTo(activityResultRegistry)
        setContent {
            TabFinderTheme(isSystemInDarkTheme()) { TabFinder(tabfinder.core, appName()) }
        }
    }

    private fun appName() = applicationInfo.loadLabel(packageManager).toString()
}
