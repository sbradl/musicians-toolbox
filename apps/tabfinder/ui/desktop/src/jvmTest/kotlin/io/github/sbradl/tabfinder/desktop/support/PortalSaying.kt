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

fun TestSessionBus.portalSaying(appearance: DesktopAppearance) = FakePortal(address, appearance)

class FakePortal(
    busAddress: String,
    appearance: DesktopAppearance,
) : AutoCloseable {
    private val settings = PortalSettings(appearance)
    private val connection: DBusConnection = DBusConnectionBuilder.forAddress(busAddress).build()

    init {
        connection.exportObject(settings)
        connection.requestBusName("org.freedesktop.portal.Desktop")
    }

    fun desktopSwitchesTo(appearance: DesktopAppearance) {
        settings.appearance = appearance
        connection.sendMessage(Settings.SettingChanged(settings.objectPath, NAMESPACE, KEY, appearance.asVariant()))
    }

    override fun close() = connection.close()
}

private class PortalSettings(
    var appearance: DesktopAppearance,
) : Settings {
    override fun getObjectPath() = "/org/freedesktop/portal/desktop"

    override fun readOne(
        namespace: String,
        key: String,
    ): Variant<*> {
        if (namespace != NAMESPACE || key != KEY) {
            throw DBusExecutionException("Requested setting not found")
        }

        return appearance.asVariant()
    }
}

private fun DesktopAppearance.asVariant() = Variant(UInt32(colorScheme.toLong()))

private const val NAMESPACE = "org.freedesktop.appearance"
private const val KEY = "color-scheme"
