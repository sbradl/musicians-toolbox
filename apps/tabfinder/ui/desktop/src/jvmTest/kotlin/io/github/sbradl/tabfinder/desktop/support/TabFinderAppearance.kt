package io.github.sbradl.tabfinder.desktop.support

import io.github.sbradl.tabfinder.desktop.PortalAppearance
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout
import kotlinx.coroutines.withTimeoutOrNull
import org.freedesktop.dbus.connections.impl.DBusConnectionBuilder
import kotlin.time.Duration.Companion.seconds

fun TestSessionBus.tabFinderAppearance() = PortalAppearance { DBusConnectionBuilder.forAddress(address).build() }

fun PortalAppearance.prefersDarkWhenTheWindowOpens(): Boolean =
    whileTabFinderRuns { scope -> withTimeout(3.seconds) { followPrefersDark(scope).value } }

fun PortalAppearance.turnsDarkAfter(change: () -> Unit): Boolean =
    whileTabFinderRuns { scope ->
        val prefersDark = followPrefersDark(scope)
        change()
        val turnedDark = withTimeoutOrNull(5.seconds) { prefersDark.first { it } }

        turnedDark != null
    }

private fun <T> whileTabFinderRuns(use: suspend (CoroutineScope) -> T): T {
    val scope = CoroutineScope(Dispatchers.IO)
    try {
        return runBlocking { use(scope) }
    } finally {
        scope.cancel()
    }
}
