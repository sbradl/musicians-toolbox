import org.gradle.api.DefaultTask
import org.gradle.api.GradleException
import org.gradle.api.file.RegularFileProperty
import org.gradle.api.provider.Property
import org.gradle.api.tasks.Input
import org.gradle.api.tasks.InputFile
import org.gradle.api.tasks.PathSensitive
import org.gradle.api.tasks.PathSensitivity
import org.gradle.api.tasks.TaskAction
import java.io.File

abstract class CheckAndroidOnlyLibrariesHaveNotices : DefaultTask() {
    @get:InputFile
    @get:PathSensitive(PathSensitivity.RELATIVE)
    abstract val appLockfile: RegularFileProperty

    @get:Input
    abstract val appRuntimeClasspath: Property<String>

    @get:InputFile
    @get:PathSensitive(PathSensitivity.RELATIVE)
    abstract val sharedLockfile: RegularFileProperty

    @get:Input
    abstract val sharedRuntimeClasspath: Property<String>

    @get:InputFile
    @get:PathSensitive(PathSensitivity.RELATIVE)
    abstract val androidOnlyLibraries: RegularFileProperty

    @TaskAction
    fun check() {
        val appLibraries = librariesIn(appLockfile.get().asFile, appRuntimeClasspath.get())
        val sharedLibraries = librariesIn(sharedLockfile.get().asFile, sharedRuntimeClasspath.get())
        val androidOnly = appLibraries - sharedLibraries
        val listed = listedLibraries()
        val unlisted = androidOnly - listed
        val stale = listed - androidOnly

        if (unlisted.isNotEmpty()) {
            throw GradleException("Add a notice to ${androidOnlyLibraries.get()} for: ${unlisted.joinToString()}")
        }
        if (stale.isNotEmpty()) {
            throw GradleException("Remove from ${androidOnlyLibraries.get()}, the app no longer ships: ${stale.joinToString()}")
        }
    }

    private fun listedLibraries(): Set<String> =
        androidOnlyLibraries
            .get()
            .asFile
            .readLines()
            .filter(String::isNotBlank)
            .map { line -> line.substringBefore(" ") }
            .toSet()

    private fun librariesIn(
        lockfile: File,
        classpath: String,
    ): Set<String> =
        lockfile
            .readLines()
            .filter { line -> classpath in line.substringAfter("=").split(",") }
            .map { line -> line.substringBeforeLast(":") }
            .toSet()
}
