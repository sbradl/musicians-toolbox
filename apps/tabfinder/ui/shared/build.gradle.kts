plugins {
    id("mt.compose-multiplatform")
    id("mt.uniffi")
}

uniffi {
    crate = "mt-tabfinder-ffi"
}

kotlin {
    sourceSets {
        jvmMain.dependencies {
            implementation(libs.compose.material3)
        }
        jvmTest.dependencies {
            implementation(libs.compose.ui.test)
            implementation(libs.junit)
            implementation(kotlin("test"))
            implementation(compose.desktop.currentOs)
        }
    }
}
