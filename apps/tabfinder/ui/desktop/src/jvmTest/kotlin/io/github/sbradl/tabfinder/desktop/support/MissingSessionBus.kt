package io.github.sbradl.tabfinder.desktop.support

import org.freedesktop.dbus.connections.impl.DBusConnection
import org.freedesktop.dbus.connections.impl.DBusConnectionBuilder

private const val GIVE_UP_AFTER_MILLISECONDS = 200
private const val SLOW_TO_GIVE_UP_MILLISECONDS = 3_000L

fun connectToAMissingSessionBus(): DBusConnection =
    DBusConnectionBuilder
        .forAddress("unix:path=/nonexistent/bus")
        .transportConfig()
        .withTimeout(GIVE_UP_AFTER_MILLISECONDS)
        .back()
        .build()

fun connectToAMissingSessionBusSlowToGiveUp(): DBusConnection {
    Thread.sleep(SLOW_TO_GIVE_UP_MILLISECONDS)

    return connectToAMissingSessionBus()
}
