plugins {
    id("mt.kotlin-multiplatform")
    id("com.android.kotlin.multiplatform.library")
    id("mt.android-lint-tool")
}

kotlin {
    android {
        compileSdk = androidSdkLevel("android-compileSdk")
        minSdk = androidSdkLevel("android-minSdk")
    }
}

licenseReport {
    configurations += "androidRuntimeClasspath"
}
