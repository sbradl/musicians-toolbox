plugins {
    id("org.jetbrains.kotlin.multiplatform")
}

interface UniffiExtension {
    val crate: Property<String>
}

val uniffi = extensions.create<UniffiExtension>("uniffi")
val workspace: Directory = rootProject.layout.projectDirectory
val nativeLibrary = uniffi.crate.map { workspace.file("target/debug/lib${it.replace('-', '_')}.so") }
val jnaResourcePrefixForLinuxX8664 = "linux-x86-64"

val cargoBuild =
    tasks.register<Exec>("cargoBuild") {
        workingDir = workspace.asFile
        commandLine("cargo", "build", "--locked", "--package")
        argumentProviders.add { listOf(uniffi.crate.get()) }
        outputs.file(nativeLibrary)
        outputs.upToDateWhen { false }
    }

val generateUniffiBindings =
    tasks.register<Exec>("generateUniffiBindings") {
        val outputDirectory = layout.buildDirectory.dir("generated/uniffi/kotlin")
        workingDir = workspace.asFile
        inputs.files(cargoBuild)
        outputs.dir(outputDirectory)
        commandLine("cargo", "run", "--locked", "--quiet", "--package", "mt-uniffi-bindgen", "--")
        argumentProviders.add {
            listOf(
                "generate",
                "--library",
                nativeLibrary.get().asFile.path,
                "--language",
                "kotlin",
                "--no-format",
                "--out-dir",
                outputDirectory.get().asFile.path,
            )
        }
        doFirst { delete(outputDirectory) }
    }

val bundleNativeLibrary =
    tasks.register<Sync>("bundleNativeLibrary") {
        from(cargoBuild) { into(jnaResourcePrefixForLinuxX8664) }
        into(layout.buildDirectory.dir("generated/uniffi/resources"))
    }

kotlin.sourceSets.named("jvmMain") {
    kotlin.srcDir(generateUniffiBindings)
    resources.srcDir(bundleNativeLibrary)
    dependencies {
        implementation(versionCatalogs.named("libs").findLibrary("jna").get())
    }
}
