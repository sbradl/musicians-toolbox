// Generated UniFFI bindings get their own module without allWarningsAsErrors/extraWarnings: we don't control their code.
import com.android.build.api.variant.KotlinMultiplatformAndroidComponentsExtension

plugins {
    id("mt.kotlin-multiplatform-android")
}

interface UniffiExtension {
    val crate: Property<String>
}

val uniffi = extensions.create<UniffiExtension>("uniffi")
val workspace: Directory = rootProject.layout.projectDirectory
val linuxTarget = "x86_64-unknown-linux-gnu"
val glibcOfOldestSupportedLinux = KotlinVersion(2, 31)
val linuxTargetWithGlibc = "$linuxTarget.${glibcOfOldestSupportedLinux.major}.${glibcOfOldestSupportedLinux.minor}"
val rustLibrary = uniffi.crate.map { crate -> workspace.file("target/$linuxTarget/debug/${linuxLibraryFileName(crate)}") }
val jnaFolderForLinuxX8664 = "linux-x86-64"
val minSdk = androidSdkLevel("android-minSdk").toString()

fun linuxLibraryFileName(crate: String) = "lib${crate.replace('-', '_')}.so"

val buildRustLibrary =
    tasks.register<Exec>("cargoBuild") {
        workingDir = workspace.asFile
        commandLine("cargo", "zigbuild", "--locked", "--target", linuxTargetWithGlibc, "--target-dir", "target", "--package")
        argumentProviders.add { listOf(uniffi.crate.get()) }
        outputs.file(rustLibrary)
        outputs.upToDateWhen { false } // Cargo decides itself what to rebuild
    }

abstract class CheckGlibcFloor : DefaultTask() {
    @get:InputFile
    abstract val library: RegularFileProperty

    @get:Input
    abstract val oldestSupportedGlibc: Property<KotlinVersion>

    @TaskAction
    fun check() {
        val newestGlibc = newestGlibcVersionIn(library.get().asFile.readText(Charsets.ISO_8859_1))
        val oldestGlibc = oldestSupportedGlibc.get()

        if (newestGlibc > oldestGlibc) {
            throw GradleException("The desktop apps must start on glibc $oldestGlibc: ${library.get()} needs glibc $newestGlibc")
        }
    }

    private fun newestGlibcVersionIn(binary: String) =
        Regex("""GLIBC_(\d+)\.(\d+)""")
            .findAll(binary)
            .map { version -> KotlinVersion(version.groupValues[1].toInt(), version.groupValues[2].toInt()) }
            .maxOrNull()
            ?: throw GradleException("${library.get()} names no glibc version: is it still a Linux library?")
}

val checkGlibcFloor =
    tasks.register<CheckGlibcFloor>("checkGlibcFloor") {
        library.set(rustLibrary)
        oldestSupportedGlibc.set(glibcOfOldestSupportedLinux)
        dependsOn(buildRustLibrary)
    }

tasks.named("check") { dependsOn(checkGlibcFloor) }

val generateKotlinBindings =
    tasks.register<Exec>("generateUniffiBindings") {
        val outputDirectory = layout.buildDirectory.dir("generated/uniffi/kotlin")
        workingDir = workspace.asFile
        inputs.files(buildRustLibrary)
        outputs.dir(outputDirectory)
        commandLine("cargo", "run", "--locked", "--quiet", "--package", "mt-uniffi-bindgen", "--")
        argumentProviders.add {
            listOf(
                "generate",
                "--library",
                rustLibrary.get().asFile.path,
                "--language",
                "kotlin",
                "--no-format",
                "--out-dir",
                outputDirectory.get().asFile.path,
            )
        }
        doFirst { delete(outputDirectory) }
    }

val bundleRustLibrary =
    tasks.register<Sync>("bundleNativeLibrary") {
        from(buildRustLibrary) { into(jnaFolderForLinuxX8664) }
        into(layout.buildDirectory.dir("generated/uniffi/resources"))
    }

abstract class CargoNdkBuild : Exec() {
    @get:OutputDirectory
    abstract val jniLibsDirectory: DirectoryProperty
}

val buildRustLibraryForAndroid =
    tasks.register<CargoNdkBuild>("cargoNdkBuild") {
        workingDir = workspace.asFile
        outputs.upToDateWhen { false } // Cargo decides itself what to rebuild
        commandLine("cargo", "ndk", "--target", "arm64-v8a", "--target", "x86_64", "--platform", minSdk, "--output-dir")
        argumentProviders.add {
            listOf(jniLibsDirectory.get().asFile.path, "build", "--locked", "--target-dir", "target", "--package", uniffi.crate.get())
        }
    }

extensions.configure<KotlinMultiplatformAndroidComponentsExtension> {
    onVariants { variant ->
        variant.sources.jniLibs?.addGeneratedSourceDirectory(buildRustLibraryForAndroid, CargoNdkBuild::jniLibsDirectory)
    }
}

val jna = versionCatalogs.named("libs").findLibrary("jna").get()

kotlin.sourceSets.named("commonMain") {
    kotlin.srcDir(generateKotlinBindings)
}

kotlin.sourceSets.named("jvmMain") {
    resources.srcDir(bundleRustLibrary)
    dependencies {
        api(jna)
    }
}

dependencies {
    "androidMainApi"(jna) { artifact { type = "aar" } }
}
