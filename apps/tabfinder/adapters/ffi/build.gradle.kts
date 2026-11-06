plugins {
    id("mt.uniffi")
}

uniffi {
    crate = "mt-tabfinder-ffi"
}

kotlin {
    android {
        namespace = "io.github.sbradl.tabfinder.core"
    }
}
