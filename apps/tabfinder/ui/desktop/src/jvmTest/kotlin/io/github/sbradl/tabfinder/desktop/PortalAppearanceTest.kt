package io.github.sbradl.tabfinder.desktop

import io.github.sbradl.tabfinder.desktop.support.DesktopAppearance
import io.github.sbradl.tabfinder.desktop.support.TestSessionBus
import io.github.sbradl.tabfinder.desktop.support.portalSaying
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.take
import kotlinx.coroutines.flow.toList
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout
import org.freedesktop.dbus.connections.impl.DBusConnectionBuilder
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue
import kotlin.time.Duration.Companion.seconds

class PortalAppearanceTest {
    @Test
    fun aDarkDesktopPrefersDark() =
        TestSessionBus().use { sessionBus ->
            sessionBus.portalSaying(DesktopAppearance.DARK).use {
                val appearance = PortalAppearance { DBusConnectionBuilder.forAddress(sessionBus.address).build() }

                assertTrue(appearance.prefersDarkAsItChanges().firstWithin())
            }
        }

    @Test
    fun aLightDesktopDoesNotPreferDark() =
        TestSessionBus().use { sessionBus ->
            sessionBus.portalSaying(DesktopAppearance.LIGHT).use {
                val appearance = PortalAppearance { DBusConnectionBuilder.forAddress(sessionBus.address).build() }

                assertFalse(appearance.prefersDarkAsItChanges().firstWithin())
            }
        }

    @Test
    fun aDesktopWithoutPreferenceDoesNotPreferDark() =
        TestSessionBus().use { sessionBus ->
            sessionBus.portalSaying(DesktopAppearance.NO_PREFERENCE).use {
                val appearance = PortalAppearance { DBusConnectionBuilder.forAddress(sessionBus.address).build() }

                assertFalse(appearance.prefersDarkAsItChanges().firstWithin())
            }
        }

    @Test
    fun aDesktopSwitchingToDarkWhileTabFinderRunsPrefersDark() =
        TestSessionBus().use { sessionBus ->
            sessionBus.portalSaying(DesktopAppearance.LIGHT).use { portal ->
                val appearance = PortalAppearance { DBusConnectionBuilder.forAddress(sessionBus.address).build() }

                val prefersDark =
                    runBlocking {
                        withTimeout(5.seconds) {
                            appearance
                                .prefersDarkAsItChanges()
                                .onEach { portal.desktopSwitchesTo(DesktopAppearance.DARK) }
                                .take(2)
                                .toList()
                        }
                    }

                assertEquals(listOf(false, true), prefersDark)
            }
        }

    @Test
    fun withoutAPortalNothingPrefersDark() =
        TestSessionBus().use { sessionBus ->
            val appearance = PortalAppearance { DBusConnectionBuilder.forAddress(sessionBus.address).build() }

            assertFalse(appearance.prefersDarkAsItChanges().firstWithin())
        }

    @Test
    fun withoutASessionBusNothingPrefersDark() {
        val appearance = PortalAppearance { DBusConnectionBuilder.forAddress("unix:path=/nonexistent/bus").build() }

        assertFalse(appearance.prefersDarkAsItChanges().firstWithin())
    }

    @Test
    fun aDarkDesktopIsKnownBeforeTheWindowOpens() =
        TestSessionBus().use { sessionBus ->
            sessionBus.portalSaying(DesktopAppearance.DARK).use {
                val appearance = PortalAppearance { DBusConnectionBuilder.forAddress(sessionBus.address).build() }

                val prefersDark = whileTabFinderRuns { scope -> appearance.followPrefersDark(scope).value }

                assertTrue(prefersDark)
            }
        }

    @Test
    fun withoutASessionBusTheWindowOpensLightWithoutWaitingForTheBus() {
        val appearance = PortalAppearance { DBusConnectionBuilder.forAddress("unix:path=/nonexistent/bus").build() }

        val prefersDark = whileTabFinderRuns { scope -> withTimeout(3.seconds) { appearance.followPrefersDark(scope).value } }

        assertFalse(prefersDark)
    }

    private fun Flow<Boolean>.firstWithin(): Boolean = runBlocking { withTimeout(20.seconds) { first() } }

    private fun <T> whileTabFinderRuns(use: suspend (CoroutineScope) -> T): T {
        val scope = CoroutineScope(Dispatchers.IO)
        try {
            return runBlocking { use(scope) }
        } finally {
            scope.cancel()
        }
    }
}
