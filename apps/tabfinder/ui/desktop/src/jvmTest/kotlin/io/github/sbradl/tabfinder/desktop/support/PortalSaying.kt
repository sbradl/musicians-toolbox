package io.github.sbradl.tabfinder.desktop.support

import io.github.sbradl.tabfinder.desktop.PORTAL_BUS_NAME
import io.github.sbradl.tabfinder.desktop.PORTAL_OBJECT_PATH
import io.github.sbradl.tabfinder.desktop.PortalSettings
import org.freedesktop.dbus.annotations.DBusInterfaceName
import org.freedesktop.dbus.annotations.DBusMemberName
import org.freedesktop.dbus.connections.impl.DBusConnection
import org.freedesktop.dbus.connections.impl.DBusConnectionBuilder
import org.freedesktop.dbus.exceptions.DBusExecutionException
import org.freedesktop.dbus.interfaces.DBusInterface
import org.freedesktop.dbus.types.UInt32
import org.freedesktop.dbus.types.Variant

enum class DesktopAppearance(
    val colorScheme: Int,
) {
    NO_PREFERENCE(0),
    DARK(1),
    LIGHT(2),
}

fun TestSessionBus.portalSaying(appearance: DesktopAppearance) = FakePortal(address, SettingsWithReadOne(appearance))

fun TestSessionBus.portalBeforeReadOneSaying(appearance: DesktopAppearance) = FakePortal(address, SettingsBeforeReadOne(appearance))

class FakePortal(
    busAddress: String,
    private val settings: FakeSettings,
) : AutoCloseable {
    private val connection: DBusConnection = DBusConnectionBuilder.forAddress(busAddress).build()

    init {
        connection.exportObject(settings)
        connection.requestBusName(PORTAL_BUS_NAME)
    }

    fun desktopSwitchesTo(appearance: DesktopAppearance) {
        settings.appearance = appearance
        connection.sendMessage(PortalSettings.SettingChanged(PORTAL_OBJECT_PATH, NAMESPACE, KEY, appearance.asVariant()))
    }

    override fun close() = connection.close()
}

abstract class FakeSettings(
    var appearance: DesktopAppearance,
) : DBusInterface {
    override fun getObjectPath() = PORTAL_OBJECT_PATH

    protected fun colorScheme(
        namespace: String,
        key: String,
    ): Variant<*> {
        if (namespace != NAMESPACE || key != KEY) {
            throw DBusExecutionException("Requested setting not found")
        }

        return appearance.asVariant()
    }
}

@DBusInterfaceName("org.freedesktop.portal.Settings")
interface SettingsWithOnlyRead : DBusInterface {
    @DBusMemberName("Read")
    fun read(
        namespace: String,
        key: String,
    ): Variant<*>
}

private class SettingsWithReadOne(
    appearance: DesktopAppearance,
) : FakeSettings(appearance),
    PortalSettings {
    override fun readOne(
        namespace: String,
        key: String,
    ) = colorScheme(namespace, key)

    override fun read(
        namespace: String,
        key: String,
    ) = Variant(colorScheme(namespace, key))
}

private class SettingsBeforeReadOne(
    appearance: DesktopAppearance,
) : FakeSettings(appearance),
    SettingsWithOnlyRead {
    override fun read(
        namespace: String,
        key: String,
    ) = Variant(colorScheme(namespace, key))
}

private fun DesktopAppearance.asVariant() = Variant(UInt32(colorScheme.toLong()))

private const val NAMESPACE = "org.freedesktop.appearance"
private const val KEY = "color-scheme"
