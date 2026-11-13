package io.github.sbradl.tabfinder.desktop

import org.freedesktop.dbus.annotations.DBusInterfaceName
import org.freedesktop.dbus.annotations.DBusMemberName
import org.freedesktop.dbus.connections.impl.DBusConnection
import org.freedesktop.dbus.connections.impl.DBusConnectionBuilder
import org.freedesktop.dbus.exceptions.DBusException
import org.freedesktop.dbus.exceptions.DBusExecutionException
import org.freedesktop.dbus.interfaces.DBusInterface
import org.freedesktop.dbus.types.UInt32
import org.freedesktop.dbus.types.Variant

class PortalAppearance(
    private val connectToSessionBus: () -> DBusConnection = { DBusConnectionBuilder.forSessionBus().build() },
) {
    fun prefersDark(): Boolean =
        try {
            connectToSessionBus().use { connection -> colorSchemeOf(connection) == COLOR_SCHEME_DARK }
        } catch (_: DBusException) {
            false
        } catch (_: DBusExecutionException) {
            false
        }

    private fun colorSchemeOf(connection: DBusConnection): Int? {
        val settings = connection.getRemoteObject(PORTAL_BUS_NAME, PORTAL_OBJECT_PATH, Settings::class.java)
        val colorScheme = settings.readOne("org.freedesktop.appearance", "color-scheme").value

        return (colorScheme as? UInt32)?.toInt()
    }

    private companion object {
        const val PORTAL_BUS_NAME = "org.freedesktop.portal.Desktop"
        const val PORTAL_OBJECT_PATH = "/org/freedesktop/portal/desktop"
        const val COLOR_SCHEME_DARK = 1
    }
}

@DBusInterfaceName("org.freedesktop.portal.Settings")
interface Settings : DBusInterface {
    @DBusMemberName("ReadOne")
    fun readOne(
        namespace: String,
        key: String,
    ): Variant<*>
}
