package io.github.sbradl.tabfinder.support

import io.github.sbradl.tabfinder.core.LibraryFolder
import io.github.sbradl.tabfinder.core.Song
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit

class LibraryFolderThatCanBeHeld(
    private val folder: LibraryFolder,
) : LibraryFolder {
    @Volatile
    private var hold = CountDownLatch(0)

    fun holdReading() {
        hold = CountDownLatch(1)
    }

    fun releaseReading() {
        hold.countDown()
    }

    override fun songs(location: String): List<Song> {
        hold.await(GIVE_UP_AFTER_SECONDS, TimeUnit.SECONDS)

        return folder.songs(location)
    }

    private companion object {
        const val GIVE_UP_AFTER_SECONDS = 5L
    }
}
