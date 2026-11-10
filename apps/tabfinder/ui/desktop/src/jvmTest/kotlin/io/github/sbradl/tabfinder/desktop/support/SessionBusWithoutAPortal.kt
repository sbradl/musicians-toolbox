package io.github.sbradl.tabfinder.desktop.support

import java.nio.file.Path
import kotlin.io.path.createTempFile
import kotlin.io.path.deleteIfExists
import kotlin.io.path.writeText

class SessionBusWithoutAPortal : AutoCloseable {
    private val config = busConfigThatStartsNoServices()
    private val daemon =
        ProcessBuilder("dbus-daemon", "--config-file=$config", "--nofork", "--print-address")
            .redirectError(ProcessBuilder.Redirect.INHERIT)
            .start()

    val address: String = daemon.inputReader().readLine()

    override fun close() {
        daemon.destroy()
        config.deleteIfExists()
    }

    private fun busConfigThatStartsNoServices(): Path {
        val config = createTempFile("session-bus", ".conf")
        config.writeText(
            """
            <busconfig>
              <type>session</type>
              <listen>unix:tmpdir=/tmp</listen>
              <policy context="default">
                <allow send_destination="*"/>
                <allow own="*"/>
              </policy>
            </busconfig>
            """.trimIndent(),
        )

        return config
    }
}
