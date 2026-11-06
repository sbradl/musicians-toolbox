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
    implementation(libs.compose.material3)
    androidTestImplementation(libs.compose.ui.test)
    androidTestImplementation(libs.androidx.test.runner)
    androidTestImplementation(libs.junit)
    androidTestImplementation(kotlin("test-junit"))
    debugImplementation(libs.compose.ui.test.manifest)
}
