import org.gradle.api.Project
import org.gradle.api.artifacts.VersionCatalogsExtension
import org.gradle.kotlin.dsl.getByType

fun Project.androidSdkLevel(name: String): Int {
    val libs = extensions.getByType<VersionCatalogsExtension>().named("libs")

    return libs
        .findVersion(name)
        .get()
        .requiredVersion
        .toInt()
}
