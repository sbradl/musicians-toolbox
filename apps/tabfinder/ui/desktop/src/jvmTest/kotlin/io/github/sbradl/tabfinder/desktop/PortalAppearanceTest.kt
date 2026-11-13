package io.github.sbradl.tabfinder.desktop

import io.github.sbradl.tabfinder.desktop.support.DesktopAppearance
import io.github.sbradl.tabfinder.desktop.support.TestSessionBus
import io.github.sbradl.tabfinder.desktop.support.connectToAMissingSessionBus
import io.github.sbradl.tabfinder.desktop.support.connectToAMissingSessionBusSlowToGiveUp
import io.github.sbradl.tabfinder.desktop.support.portalBeforeReadOneSaying
import io.github.sbradl.tabfinder.desktop.support.portalSaying
import io.github.sbradl.tabfinder.desktop.support.prefersDarkWhenTheWindowOpens
import io.github.sbradl.tabfinder.desktop.support.tabFinderAppearance
import io.github.sbradl.tabfinder.desktop.support.turnsDarkAfter
import kotlin.test.Test
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class PortalAppearanceTest {
    @Test
    fun aDarkDesktopOpensTabFinderDark() =
        TestSessionBus().use { sessionBus ->
            sessionBus.portalSaying(DesktopAppearance.DARK).use {
                assertTrue(sessionBus.tabFinderAppearance().prefersDarkWhenTheWindowOpens())
            }
        }

    @Test
    fun aLightDesktopOpensTabFinderLight() =
        TestSessionBus().use { sessionBus ->
            sessionBus.portalSaying(DesktopAppearance.LIGHT).use {
                assertFalse(sessionBus.tabFinderAppearance().prefersDarkWhenTheWindowOpens())
            }
        }

    @Test
    fun aDesktopWithoutPreferenceOpensTabFinderLight() =
        TestSessionBus().use { sessionBus ->
            sessionBus.portalSaying(DesktopAppearance.NO_PREFERENCE).use {
                assertFalse(sessionBus.tabFinderAppearance().prefersDarkWhenTheWindowOpens())
            }
        }

    @Test
    fun aDarkDesktopWithAnOlderPortalOpensTabFinderDark() =
        TestSessionBus().use { sessionBus ->
            sessionBus.portalBeforeReadOneSaying(DesktopAppearance.DARK).use {
                assertTrue(sessionBus.tabFinderAppearance().prefersDarkWhenTheWindowOpens())
            }
        }

    @Test
    fun aDesktopSwitchingToDarkWhileTabFinderRunsTurnsItDark() =
        TestSessionBus().use { sessionBus ->
            sessionBus.portalSaying(DesktopAppearance.LIGHT).use { portal ->
                val appearance = sessionBus.tabFinderAppearance()

                val turnedDark = appearance.turnsDarkAfter { portal.desktopSwitchesTo(DesktopAppearance.DARK) }

                assertTrue(turnedDark)
            }
        }

    @Test
    fun withoutAPortalTabFinderOpensLight() =
        TestSessionBus().use { sessionBus ->
            assertFalse(sessionBus.tabFinderAppearance().prefersDarkWhenTheWindowOpens())
        }

    @Test
    fun withoutASessionBusTabFinderOpensLight() {
        val appearance = PortalAppearance(::connectToAMissingSessionBus)

        assertFalse(appearance.prefersDarkWhenTheWindowOpens())
    }

    @Test
    fun aSessionBusSlowToGiveUpOpensTabFinderLightWithoutWaitingForIt() {
        val appearance = PortalAppearance(::connectToAMissingSessionBusSlowToGiveUp)

        assertFalse(appearance.prefersDarkWhenTheWindowOpens())
    }
}
