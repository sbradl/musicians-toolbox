package io.github.sbradl.tabfinder

import android.app.Application
import androidx.lifecycle.AndroidViewModel
import io.github.sbradl.tabfinder.ui.TabFinderCore

class RunningTabFinder(
    app: Application,
) : AndroidViewModel(app) {
    val folderPicker = SafFolderPicker(app.contentResolver)
    val core = TabFinderCore.withLibraryFolder(folderPicker, app.filesDir.path, SafLibraryFolder(app.contentResolver))

    init {
        core.start()
    }
}
