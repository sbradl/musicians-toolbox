package io.github.sbradl.tabfinder.support

import android.app.Activity
import android.os.Bundle
import androidx.activity.result.ActivityResultRegistry
import androidx.activity.result.contract.ActivityResultContract
import androidx.core.app.ActivityOptionsCompat
import kotlinx.coroutines.CompletableDeferred

class SystemFolderPickerOnAScreen {
    private var openedRequest: Int? = null
    private val opened = CompletableDeferred<Unit>()

    val registry =
        object : ActivityResultRegistry() {
            override fun <I, O> onLaunch(
                requestCode: Int,
                contract: ActivityResultContract<I, O>,
                input: I,
                options: ActivityOptionsCompat?,
            ) {
                openedRequest = requestCode
                opened.complete(Unit)
            }
        }

    suspend fun awaitPickerOpened() = opened.await()

    fun turned(): SystemFolderPickerOnAScreen {
        val savedScreen = Bundle()
        registry.onSaveInstanceState(savedScreen)
        val screenAfter = SystemFolderPickerOnAScreen()
        screenAfter.openedRequest = openedRequest
        screenAfter.registry.onRestoreInstanceState(savedScreen)

        return screenAfter
    }

    fun musicianClosesThePicker() {
        val request = requireNotNull(openedRequest) { "The system folder picker was never opened" }
        registry.dispatchResult(request, Activity.RESULT_CANCELED, null)
    }
}
