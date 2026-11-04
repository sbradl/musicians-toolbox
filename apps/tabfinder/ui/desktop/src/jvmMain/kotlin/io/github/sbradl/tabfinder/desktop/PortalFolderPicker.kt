package io.github.sbradl.tabfinder.desktop

import io.github.sbradl.tabfinder.ui.FolderPicker
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.freedesktop.dbus.DBusPath
import org.freedesktop.dbus.annotations.DBusInterfaceName
import org.freedesktop.dbus.annotations.DBusMemberName
import org.freedesktop.dbus.connections.impl.DBusConnection
import org.freedesktop.dbus.connections.impl.DBusConnectionBuilder
import org.freedesktop.dbus.exceptions.DBusException
import org.freedesktop.dbus.interfaces.DBusInterface
import org.freedesktop.dbus.interfaces.DBusSigHandler
import org.freedesktop.dbus.messages.DBusSignal
import org.freedesktop.dbus.types.UInt32
import org.freedesktop.dbus.types.Variant
import java.net.URI
import java.nio.file.Path
import java.util.UUID

class PortalFolderPicker : FolderPicker {
    override suspend fun pickFolder(): String? =
        withContext(Dispatchers.IO) {
            try {
                DBusConnectionBuilder.forSessionBus().build().use { connection -> pickFolder(connection) }
            } catch (_: DBusException) {
                null
            }
        }

    private suspend fun pickFolder(connection: DBusConnection): String? {
        val token = "tabfinder_" + UUID.randomUUID().toString().replace('-', '_')
        val sender = connection.uniqueName.removePrefix(":").replace('.', '_')
        val requestPath = "/org/freedesktop/portal/desktop/request/$sender/$token"
        val response = CompletableDeferred<Request.Response>()
        val handler = DBusSigHandler<Request.Response> { if (it.path == requestPath) response.complete(it) }
        connection.addSigHandler(Request.Response::class.java, handler).use {
            connection
                .getRemoteObject(PORTAL_BUS_NAME, PORTAL_OBJECT_PATH, FileChooser::class.java)
                .openFile(
                    "",
                    "",
                    mapOf(
                        "handle_token" to Variant(token),
                        "directory" to Variant(true),
                        "modal" to Variant(true),
                    ),
                )
            return response.await().pickedFolder()
        }
    }

    private fun Request.Response.pickedFolder(): String? {
        val uris = results["uris"]?.value
        val uri = (uris as? List<*>)?.firstOrNull() ?: (uris as? Array<*>)?.firstOrNull()
        return if (response.toInt() == RESPONSE_SUCCESS && uri is String) localPath(uri) else null
    }

    private fun localPath(uri: String): String? = runCatching { Path.of(URI(uri)).toString() }.getOrNull()

    private companion object {
        const val PORTAL_BUS_NAME = "org.freedesktop.portal.Desktop"
        const val PORTAL_OBJECT_PATH = "/org/freedesktop/portal/desktop"
        const val RESPONSE_SUCCESS = 0
    }
}

@DBusInterfaceName("org.freedesktop.portal.FileChooser")
interface FileChooser : DBusInterface {
    @DBusMemberName("OpenFile")
    fun openFile(
        parentWindow: String,
        title: String,
        options: Map<String, @JvmSuppressWildcards Variant<*>>,
    ): DBusPath
}

@DBusInterfaceName("org.freedesktop.portal.Request")
interface Request : DBusInterface {
    @DBusMemberName("Response")
    class Response(
        path: String,
        val response: UInt32,
        val results: Map<String, @JvmSuppressWildcards Variant<*>>,
    ) : DBusSignal(path, response, results)
}
