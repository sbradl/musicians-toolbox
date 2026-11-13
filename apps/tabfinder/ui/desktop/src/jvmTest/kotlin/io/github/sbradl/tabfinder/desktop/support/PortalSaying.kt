package io.github.sbradl.tabfinder.desktop.support

import io.github.sbradl.tabfinder.desktop.Settings
import org.freedesktop.dbus.connections.impl.DBusConnection
import org.freedesktop.dbus.connections.impl.DBusConnectionBuilder
import org.freedesktop.dbus.exceptions.DBusExecutionException
import org.freedesktop.dbus.types.UInt32
import org.freedesktop.dbus.types.Variant

enum class DesktopAppearance(
    val colorScheme: Int,
) {
    NO_PREFERENCE(0),
    DARK(1),
    LIGHT(2),
}

fun TestSessionBus.portalSaying(appearance: DesktopAppearance): DBusConnection {
    val portal = DBusConnectionBuilder.forAddress(address).build()
    portal.exportObject(PortalSettings(appearance))
    portal.requestBusName("org.freedesktop.portal.Desktop")

    return portal
}

private class PortalSettings(
    private val appearance: DesktopAppearance,
) : Settings {
    override fun getObjectPath() = "/org/freedesktop/portal/desktop"

    override fun readOne(
        namespace: String,
        key: String,
    ): Variant<*> {
        if (namespace != "org.freedesktop.appearance" || key != "color-scheme") {
            throw DBusExecutionException("Requested setting not found")
        }

        return Variant(UInt32(appearance.colorScheme.toLong()))
    }
}
