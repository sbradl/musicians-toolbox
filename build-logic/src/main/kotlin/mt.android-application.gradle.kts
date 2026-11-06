import com.android.build.api.artifact.SingleArtifact
import com.android.build.api.variant.BuiltArtifactsLoader

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.plugin.compose")
    id("mt.android-lint-tool")
}

android {
    compileSdk = androidSdkLevel("android-compileSdk")
    defaultConfig {
        minSdk = androidSdkLevel("android-minSdk")
        targetSdk = 36
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        ndk { abiFilters += listOf("arm64-v8a", "x86_64") }
    }
    lint {
        warningsAsErrors = true
        // Library lint.jars built against older lint APIs (lifecycle-runtime via Compose) are skipped anyway; user decision
        disable += "ObsoleteLintCustomCheck"
    }
}

kotlin {
    jvmToolchain(21)
    compilerOptions {
        allWarningsAsErrors.set(true)
        extraWarnings.set(true)
    }
}

dependencyLocking {
    lockAllConfigurations()
}

abstract class CheckNoInternetPermission : DefaultTask() {
    @get:InputFile
    abstract val mergedManifest: RegularFileProperty

    @TaskAction
    fun check() {
        val requestsInternet = "android.permission.INTERNET" in mergedManifest.get().asFile.readText()

        if (requestsInternet) {
            throw GradleException("The apps are offline (ADR 0005): the merged manifest must not request android.permission.INTERNET")
        }
    }
}

abstract class CheckSixteenKbAlignment : DefaultTask() {
    @get:InputFiles
    abstract val apkFolder: DirectoryProperty

    @get:Internal
    abstract val apkLoader: Property<BuiltArtifactsLoader>

    @get:InputFile
    abstract val zipalign: RegularFileProperty

    @get:Inject
    abstract val execOperations: ExecOperations

    @TaskAction
    fun check() {
        val apks = requireNotNull(apkLoader.get().load(apkFolder.get())) { "No APK in ${apkFolder.get()}" }
        apks.elements.forEach { apk -> checkAlignment(apk.outputFile) }
    }

    private fun checkAlignment(apk: String) {
        execOperations.exec { commandLine(zipalign.get().asFile.path, "-c", "-P", "16", "4", apk) }
    }
}

val zipalignOfTheSdk =
    androidComponents.sdkComponents.sdkDirectory.map { sdk ->
        sdk.file("build-tools/${android.buildToolsVersion}/zipalign")
    }

androidComponents {
    onVariants { variant ->
        val checkNoInternetPermission =
            tasks.register<CheckNoInternetPermission>(taskName("check", variant, "NoInternetPermission")) {
                mergedManifest.set(variant.artifacts.get(SingleArtifact.MERGED_MANIFEST))
            }
        val checkSixteenKbAlignment =
            tasks.register<CheckSixteenKbAlignment>(taskName("check", variant, "SixteenKbAlignment")) {
                apkFolder.set(variant.artifacts.get(SingleArtifact.APK))
                apkLoader.set(variant.artifacts.getBuiltArtifactsLoader())
                zipalign.set(zipalignOfTheSdk)
            }
        tasks.named("check") { dependsOn(checkNoInternetPermission, checkSixteenKbAlignment) }
    }
}

fun taskName(
    verb: String,
    variant: com.android.build.api.variant.Variant,
    subject: String,
) = verb + variant.name.replaceFirstChar(Char::uppercase) + subject
