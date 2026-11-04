plugins {
    id("mt.kotlin-multiplatform")
    id("org.jetbrains.kotlin.plugin.compose")
    id("org.jetbrains.compose")
}

kotlin {
    compilerOptions {
        allWarningsAsErrors.set(true)
        extraWarnings.set(true)
    }
}
