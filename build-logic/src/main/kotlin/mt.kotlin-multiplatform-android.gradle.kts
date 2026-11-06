plugins {
    id("mt.kotlin-multiplatform")
    id("com.android.kotlin.multiplatform.library")
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
