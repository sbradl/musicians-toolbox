package io.github.sbradl.tabfinder.desktop

import io.github.sbradl.tabfinder.desktop.support.DesktopAppearance
import io.github.sbradl.tabfinder.desktop.support.TestSessionBus
import io.github.sbradl.tabfinder.desktop.support.portalSaying
import org.freedesktop.dbus.connections.impl.DBusConnectionBuilder
import kotlin.test.Test
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class PortalAppearanceTest {
    @Test
    fun aDarkDesktopPrefersDark() =
        TestSessionBus().use { sessionBus ->
            sessionBus.portalSaying(DesktopAppearance.DARK).use {
                val appearance = PortalAppearance { DBusConnectionBuilder.forAddress(sessionBus.address).build() }

                assertTrue(appearance.prefersDark())
            }
        }

    @Test
    fun aLightDesktopDoesNotPreferDark() =
        TestSessionBus().use { sessionBus ->
            sessionBus.portalSaying(DesktopAppearance.LIGHT).use {
                val appearance = PortalAppearance { DBusConnectionBuilder.forAddress(sessionBus.address).build() }

                assertFalse(appearance.prefersDark())
            }
        }

    @Test
    fun aDesktopWithoutPreferenceDoesNotPreferDark() =
        TestSessionBus().use { sessionBus ->
            sessionBus.portalSaying(DesktopAppearance.NO_PREFERENCE).use {
                val appearance = PortalAppearance { DBusConnectionBuilder.forAddress(sessionBus.address).build() }

                assertFalse(appearance.prefersDark())
            }
        }

    @Test
    fun withoutAPortalNothingPrefersDark() =
        TestSessionBus().use { sessionBus ->
            val appearance = PortalAppearance { DBusConnectionBuilder.forAddress(sessionBus.address).build() }

            assertFalse(appearance.prefersDark())
        }

    @Test
    fun withoutASessionBusNothingPrefersDark() {
        val appearance = PortalAppearance { DBusConnectionBuilder.forAddress("unix:path=/nonexistent/bus").build() }

        assertFalse(appearance.prefersDark())
    }
}
