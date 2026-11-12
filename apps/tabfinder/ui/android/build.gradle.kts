plugins {
    id("mt.android-application")
}

android {
    namespace = "io.github.sbradl.tabfinder"
    defaultConfig {
        applicationId = "io.github.sbradl.tabfinder"
    }
}

dependencies {
    implementation(project(":apps:tabfinder:ui:shared"))
    implementation(libs.activity.compose)
    implementation(libs.lifecycle.viewmodel)
    implementation(libs.compose.material3)
    androidTestImplementation(libs.compose.ui.test)
    androidTestImplementation(libs.androidx.test.runner)
    androidTestImplementation(libs.junit)
    androidTestImplementation(kotlin("test-junit"))
    debugImplementation(libs.compose.ui.test.manifest)
}

val librariesOnlyOnAndroid = layout.projectDirectory.file("android-only-libraries.txt")
val shared = project(":apps:tabfinder:ui:shared")

val checkAndroidOnlyLibrariesHaveNotices =
    tasks.register<CheckAndroidOnlyLibrariesHaveNotices>("checkAndroidOnlyLibrariesHaveNotices") {
        appLockfile.set(layout.projectDirectory.file("gradle.lockfile"))
        appRuntimeClasspath.set("releaseRuntimeClasspath")
        sharedLockfile.set(shared.layout.projectDirectory.file("gradle.lockfile"))
        sharedRuntimeClasspath.set("androidRuntimeClasspath")
        androidOnlyLibraries.set(librariesOnlyOnAndroid)
    }

tasks.named("check") { dependsOn(checkAndroidOnlyLibrariesHaveNotices) }

val thirdPartyNotices =
    tasks.register<ThirdPartyNotices>("thirdPartyNotices") {
        forRustCrate(rootProject.layout.projectDirectory, "apps/tabfinder/adapters/ffi")
        kotlinLicenseReport.set(shared.layout.buildDirectory.dir("reports/dependency-license"))
        dependsOn(":apps:tabfinder:ui:shared:generateLicenseReport")
        androidOnlyLibraries.set(librariesOnlyOnAndroid)
        outputDirectory.set(layout.buildDirectory.dir("generated/third-party-notices"))
    }

androidComponents {
    onVariants { variant ->
        variant.sources.assets?.addGeneratedSourceDirectory(thirdPartyNotices, ThirdPartyNotices::outputDirectory)
    }
}
