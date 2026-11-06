plugins {
    id("mt.compose-multiplatform")
}

kotlin {
    sourceSets {
        jvmMain.dependencies {
            implementation(project(":apps:tabfinder:ui:shared"))
            implementation(libs.compose.material3)
            implementation(compose.desktop.currentOs)
            implementation(libs.dbus.java.core)
            implementation(libs.dbus.java.transport.native.unixsocket)
        }
    }
}

compose.desktop {
    application {
        mainClass = "io.github.sbradl.tabfinder.desktop.MainKt"
        nativeDistributions {
            packageName = "TabFinder"
            packageVersion = "0.0.0-dev"
            modules("java.instrument", "jdk.security.auth", "jdk.unsupported")
        }
    }
}
