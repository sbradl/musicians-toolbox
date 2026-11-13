package io.github.sbradl.tabfinder.desktop

import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.ProducerScope
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.channelFlow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.launch
import kotlinx.coroutines.withTimeoutOrNull
import org.freedesktop.dbus.annotations.DBusInterfaceName
import org.freedesktop.dbus.annotations.DBusMemberName
import org.freedesktop.dbus.connections.impl.DBusConnection
import org.freedesktop.dbus.connections.impl.DBusConnectionBuilder
import org.freedesktop.dbus.exceptions.DBusException
import org.freedesktop.dbus.exceptions.DBusExecutionException
import org.freedesktop.dbus.interfaces.DBusInterface
import org.freedesktop.dbus.interfaces.DBusSigHandler
import org.freedesktop.dbus.messages.DBusSignal
import org.freedesktop.dbus.types.UInt32
import org.freedesktop.dbus.types.Variant
import kotlin.time.Duration.Companion.seconds

class PortalAppearance(
    private val connectToSessionBus: () -> DBusConnection = { DBusConnectionBuilder.forSessionBus().build() },
) {
    suspend fun followPrefersDark(scope: CoroutineScope): StateFlow<Boolean> {
        val prefersDark = MutableStateFlow(false)
        val firstAnswer = CompletableDeferred<Unit>()
        scope.launch {
            prefersDarkAsItChanges().collect { answer ->
                prefersDark.value = answer
                firstAnswer.complete(Unit)
            }
        }
        withTimeoutOrNull(FIRST_ANSWER_WAIT) { firstAnswer.await() }

        return prefersDark
    }

    fun prefersDarkAsItChanges(): Flow<Boolean> =
        channelFlow {
            try {
                connectToSessionBus().use { connection -> followColorScheme(connection) }
            } catch (_: DBusException) {
                send(false)
            } catch (_: DBusExecutionException) {
                send(false)
            }
        }.flowOn(Dispatchers.IO)

    private suspend fun ProducerScope<Boolean>.followColorScheme(connection: DBusConnection) {
        val listenForChanges =
            DBusSigHandler<Settings.SettingChanged> { change ->
                if (change.isColorScheme()) trySend(isDark(change.value))
            }
        connection.addSigHandler(Settings.SettingChanged::class.java, listenForChanges).use {
            send(isDark(colorSchemeOf(connection)))
            awaitClose()
        }
    }

    private fun colorSchemeOf(connection: DBusConnection): Variant<*> {
        val settings = connection.getRemoteObject(PORTAL_BUS_NAME, PORTAL_OBJECT_PATH, Settings::class.java)

        return settings.readOne(APPEARANCE_NAMESPACE, COLOR_SCHEME_KEY)
    }

    private fun Settings.SettingChanged.isColorScheme() = namespace == APPEARANCE_NAMESPACE && key == COLOR_SCHEME_KEY

    private fun isDark(colorScheme: Variant<*>) = (colorScheme.value as? UInt32)?.toInt() == COLOR_SCHEME_DARK

    private companion object {
        const val PORTAL_BUS_NAME = "org.freedesktop.portal.Desktop"
        const val PORTAL_OBJECT_PATH = "/org/freedesktop/portal/desktop"
        const val APPEARANCE_NAMESPACE = "org.freedesktop.appearance"
        const val COLOR_SCHEME_KEY = "color-scheme"
        const val COLOR_SCHEME_DARK = 1
        val FIRST_ANSWER_WAIT = 1.seconds
    }
}

@DBusInterfaceName("org.freedesktop.portal.Settings")
interface Settings : DBusInterface {
    @DBusMemberName("ReadOne")
    fun readOne(
        namespace: String,
        key: String,
    ): Variant<*>

    @DBusMemberName("SettingChanged")
    class SettingChanged(
        path: String,
        val namespace: String,
        val key: String,
        val value: Variant<*>,
    ) : DBusSignal(path, namespace, key, value)
}
