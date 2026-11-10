import groovy.json.JsonSlurper
import org.gradle.api.DefaultTask
import org.gradle.api.GradleException
import org.gradle.api.file.Directory
import org.gradle.api.file.DirectoryProperty
import org.gradle.api.file.RegularFileProperty
import org.gradle.api.tasks.InputDirectory
import org.gradle.api.tasks.InputFile
import org.gradle.api.tasks.Internal
import org.gradle.api.tasks.Optional
import org.gradle.api.tasks.OutputDirectory
import org.gradle.api.tasks.PathSensitive
import org.gradle.api.tasks.PathSensitivity
import org.gradle.api.tasks.TaskAction
import org.gradle.process.ExecOperations
import java.io.ByteArrayOutputStream
import javax.inject.Inject

abstract class ThirdPartyNotices : DefaultTask() {
    @get:Internal
    abstract val workspace: DirectoryProperty

    @get:InputFile
    @get:PathSensitive(PathSensitivity.RELATIVE)
    abstract val rustCrate: RegularFileProperty

    @get:InputFile
    @get:PathSensitive(PathSensitivity.RELATIVE)
    abstract val cargoLock: RegularFileProperty

    @get:InputFile
    @get:PathSensitive(PathSensitivity.RELATIVE)
    abstract val cargoAboutConfig: RegularFileProperty

    @get:InputDirectory
    @get:PathSensitive(PathSensitivity.RELATIVE)
    abstract val kotlinLicenseReport: DirectoryProperty

    @get:Optional
    @get:InputFile
    @get:PathSensitive(PathSensitivity.RELATIVE)
    abstract val androidOnlyLibraries: RegularFileProperty

    @get:OutputDirectory
    abstract val outputDirectory: DirectoryProperty

    @get:Inject
    abstract val execOperations: ExecOperations

    fun forRustCrate(
        workspace: Directory,
        crateFolder: String,
    ) {
        this.workspace.set(workspace)
        rustCrate.set(workspace.file("$crateFolder/Cargo.toml"))
        cargoLock.set(workspace.file("Cargo.lock"))
        cargoAboutConfig.set(workspace.file("about.toml"))
    }

    @TaskAction
    fun write() {
        val sections = listOfNotNull(rustCrates(), kotlinLibraries(), androidOnlyLibrariesSection())
        val notices = outputDirectory.file(FILE_NAME).get().asFile

        notices.writeText(sections.joinToString("\n\n"))
    }

    private fun rustCrates(): String {
        val licenses = cargoAbout()["licenses"] as List<*>
        val notices = licenses.joinToString("") { license -> rustLicenseNotice(license as Map<*, *>) }

        return "Rust crates\n===========\n$notices"
    }

    private fun cargoAbout(): Map<*, *> {
        val json = ByteArrayOutputStream()
        execOperations.exec {
            workingDir = workspace.get().asFile
            commandLine("cargo", "about", "generate", "--locked", "--format", "json")
            args("--manifest-path", rustCrate.get().asFile.path, "--config", cargoAboutConfig.get().asFile.path)
            standardOutput = json
        }

        return JsonSlurper().parseText(json.toString(Charsets.UTF_8)) as Map<*, *>
    }

    private fun rustLicenseNotice(license: Map<*, *>): String {
        val crates = (license["used_by"] as List<*>).map { usage -> crateOf(usage as Map<*, *>) }
        val usedBy = crates.joinToString("") { crate -> "- $crate\n" }

        return "\n${license["name"]} (${license["id"]}), used by:\n$usedBy\n${license["text"]}\n"
    }

    private fun crateOf(usage: Map<*, *>): String {
        val crate = usage["crate"] as Map<*, *>

        return "${crate["name"]} ${crate["version"]}"
    }

    private fun kotlinLibraries(): String {
        val report = kotlinLicenseReport.file(FILE_NAME).get().asFile

        if (!report.exists()) {
            throw GradleException("$report is missing: does the licence report still render text?")
        }

        return "Kotlin and Java libraries\n=========================\n\n" + report.readText()
    }

    private fun androidOnlyLibrariesSection(): String? {
        val libraries = androidOnlyLibraries.orNull?.asFile ?: return null

        return "Android libraries (licence texts above)\n=======================================\n\n" + libraries.readText()
    }

    companion object {
        const val FILE_NAME = "THIRD-PARTY-NOTICES.txt"
    }
}
