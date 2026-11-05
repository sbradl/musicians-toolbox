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
            withSessionBus { connection -> pickFolder(connection) }
        }

    private suspend fun pickFolder(connection: DBusConnection): String? {
        val token = uniqueRequestToken()
        val answer =
            connection.awaitAnswerOn(pathThePortalAnswersOn(connection, token)) {
                fileChooserOf(connection).openFile(parentWindow = "", title = "", options = folderPickerOptions(token))
            }
        return answer.pickedFolder()
    }

    private inline fun withSessionBus(use: (DBusConnection) -> String?): String? =
        try {
            DBusConnectionBuilder.forSessionBus().build().use(use)
        } catch (_: DBusException) {
            null
        }

    private suspend fun DBusConnection.awaitAnswerOn(
        answerPath: String,
        ask: () -> Unit,
    ): Request.Response {
        val answer = CompletableDeferred<Request.Response>()
        val listenForTheAnswer = DBusSigHandler<Request.Response> { if (it.path == answerPath) answer.complete(it) }
        addSigHandler(Request.Response::class.java, listenForTheAnswer).use {
            ask()
            return answer.await()
        }
    }

    private fun uniqueRequestToken() = "tabfinder_" + UUID.randomUUID().toString().replace('-', '_')

    private fun pathThePortalAnswersOn(
        connection: DBusConnection,
        token: String,
    ): String {
        val sender = connection.uniqueName.removePrefix(":").replace('.', '_')
        return "/org/freedesktop/portal/desktop/request/$sender/$token"
    }

    private fun fileChooserOf(connection: DBusConnection) =
        connection.getRemoteObject(PORTAL_BUS_NAME, PORTAL_OBJECT_PATH, FileChooser::class.java)

    private fun folderPickerOptions(token: String) =
        mapOf(
            "handle_token" to Variant(token),
            "directory" to Variant(true),
            "modal" to Variant(true),
        )

    private fun Request.Response.pickedFolder(): String? {
        if (response.toInt() != RESPONSE_SUCCESS) return null
        val uri = firstUri() ?: return null
        return localPath(uri)
    }

    private fun Request.Response.firstUri(): String? {
        val firstUri =
            when (val uris = results["uris"]?.value) {
                is List<*> -> uris.firstOrNull()
                is Array<*> -> uris.firstOrNull()
                else -> null
            }
        return firstUri as? String
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
