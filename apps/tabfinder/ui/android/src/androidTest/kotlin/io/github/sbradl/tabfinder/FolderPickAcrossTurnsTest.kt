package io.github.sbradl.tabfinder

import androidx.test.platform.app.InstrumentationRegistry
import io.github.sbradl.tabfinder.support.SystemFolderPickerOnAScreen
import io.github.sbradl.tabfinder.ui.FolderPick
import kotlinx.coroutines.async
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout
import kotlin.test.Test
import kotlin.test.assertEquals

class FolderPickAcrossTurnsTest {
    private val app = InstrumentationRegistry.getInstrumentation().targetContext

    @Test
    fun theAnswerOfAPickerOpenedBeforeTheScreenTurnedStillArrives() =
        runBlocking {
            val folderPicker = SafFolderPicker(app.contentResolver)
            val screenBefore = SystemFolderPickerOnAScreen()
            folderPicker.attachTo(screenBefore.registry)
            val pick = async { folderPicker.pickFolder("Choose the Library folder") }
            screenBefore.awaitPickerOpened()

            val screenAfter = screenBefore.turned()
            folderPicker.attachTo(screenAfter.registry)
            screenAfter.musicianClosesThePicker()

            assertEquals(FolderPick.Cancelled, withTimeout(GIVE_UP_AFTER_MILLIS) { pick.await() })
        }

    private companion object {
        const val GIVE_UP_AFTER_MILLIS = 5_000L
    }
}
