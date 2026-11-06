pluginManagement {
    includeBuild("build-logic")
    repositories {
        google()
        gradlePluginPortal()
    }
}

dependencyResolutionManagement {
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = "musicians-toolbox"

include(":apps:tabfinder:adapters:ffi")
include(":apps:tabfinder:ui:shared")
include(":apps:tabfinder:ui:desktop")
include(":apps:tabfinder:ui:android")
