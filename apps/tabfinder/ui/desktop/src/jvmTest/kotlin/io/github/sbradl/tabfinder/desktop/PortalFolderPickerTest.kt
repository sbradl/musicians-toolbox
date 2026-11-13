package io.github.sbradl.tabfinder.desktop

import io.github.sbradl.tabfinder.desktop.support.TestSessionBus
import io.github.sbradl.tabfinder.ui.FolderPick
import kotlinx.coroutines.runBlocking
import org.freedesktop.dbus.connections.impl.DBusConnectionBuilder
import kotlin.test.Test
import kotlin.test.assertEquals

class PortalFolderPickerTest {
    @Test
    fun withoutASessionBusThereIsNoFolderDialog() {
        val picker = PortalFolderPicker { DBusConnectionBuilder.forAddress("unix:path=/nonexistent/bus").build() }

        val pick = runBlocking { picker.pickFolder("Choose the Library folder") }

        assertEquals(FolderPick.Unavailable, pick)
    }

    @Test
    fun withoutAPortalOnTheSessionBusThereIsNoFolderDialog() =
        TestSessionBus().use { sessionBus ->
            val picker = PortalFolderPicker { DBusConnectionBuilder.forAddress(sessionBus.address).build() }

            val pick = runBlocking { picker.pickFolder("Choose the Library folder") }

            assertEquals(FolderPick.Unavailable, pick)
        }
}
