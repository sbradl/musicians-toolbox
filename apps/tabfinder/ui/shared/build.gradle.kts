plugins {
    id("mt.compose-multiplatform")
    id("mt.kotlin-multiplatform-android")
}

kotlin {
    android {
        namespace = "io.github.sbradl.tabfinder.ui"
    }
    sourceSets {
        commonMain.dependencies {
            api(project(":apps:tabfinder:adapters:ffi"))
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

dependencies {
    constraints {
        add("jvmTestImplementation", libs.guava) {
            because("Compose ui-test pulls Guava 28.1 via Truth, which has CVE-2026-102554, CVE-2023-2976 and CVE-2020-8908")
        }
    }
}
