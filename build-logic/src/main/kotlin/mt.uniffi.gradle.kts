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
val glibcOfOldestSupportedLinux = GlibcVersion(2, 31)
val linuxTargetWithGlibc = "$linuxTarget.$glibcOfOldestSupportedLinux"
val rustLibrary = uniffi.crate.map { crate -> workspace.file("target/$linuxTarget/debug/${linuxLibraryFileName(crate)}") }
val jnaFolderForLinuxX8664 = "linux-x86-64"
val minSdk = androidSdkLevel("android-minSdk").toString()
val abisToBuild = androidAbis()

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
    abstract val oldestSupportedGlibc: Property<GlibcVersion>

    @TaskAction
    fun check() {
        val newestGlibc = newestGlibcNeededBy(library.get().asFile)
        val oldestGlibc = oldestSupportedGlibc.get()

        if (newestGlibc > oldestGlibc) {
            throw GradleException("The desktop apps must start on glibc $oldestGlibc: ${library.get()} needs glibc $newestGlibc")
        }
    }

    private fun newestGlibcNeededBy(library: File): GlibcVersion {
        val everyByteAsOneCharacter = library.readText(Charsets.ISO_8859_1)
        val newestGlibc =
            Regex("""GLIBC_(\d+)\.(\d+)""")
                .findAll(everyByteAsOneCharacter)
                .map { version -> GlibcVersion(version.groupValues[1].toInt(), version.groupValues[2].toInt()) }
                .maxOrNull()

        return newestGlibc ?: throw GradleException("$library names no glibc version: is it still a Linux library?")
    }
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
        commandLine("cargo", "ndk")
        argumentProviders.add { abisToBuild.flatMap { abi -> listOf("--target", abi) } }
        argumentProviders.add {
            listOf("--platform", minSdk, "--output-dir", jniLibsDirectory.get().asFile.path)
        }
        argumentProviders.add { listOf("build", "--locked", "--target-dir", "target", "--package", uniffi.crate.get()) }
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
