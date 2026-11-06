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
val rustLibrary = uniffi.crate.map { crate -> workspace.file("target/debug/${linuxLibraryFileName(crate)}") }
val jnaFolderForLinuxX8664 = "linux-x86-64"
val minSdk = androidSdkLevel("android-minSdk").toString()

fun linuxLibraryFileName(crate: String) = "lib${crate.replace('-', '_')}.so"

val buildRustLibrary =
    tasks.register<Exec>("cargoBuild") {
        workingDir = workspace.asFile
        commandLine("cargo", "build", "--locked", "--target-dir", "target", "--package")
        argumentProviders.add { listOf(uniffi.crate.get()) }
        outputs.file(rustLibrary)
        outputs.upToDateWhen { false } // Cargo decides itself what to rebuild
    }

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
