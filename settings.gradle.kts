pluginManagement {
    includeBuild("build-logic")
}

dependencyResolutionManagement {
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = "musicians-toolbox"

include(":apps:tabfinder:ui:shared")
include(":apps:tabfinder:ui:desktop")
