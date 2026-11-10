import org.gradle.api.Project

fun Project.androidAbis(): List<String> {
    val abis = providers.gradleProperty("mtAndroidAbis").getOrElse("arm64-v8a,x86_64")

    return abis.split(",").map { abi -> abi.trim() }
}
