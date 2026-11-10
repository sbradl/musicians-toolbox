import com.github.jk1.license.render.ReportRenderer
import com.github.jk1.license.render.TextReportRenderer

plugins {
    id("org.jetbrains.kotlin.multiplatform")
    id("com.github.jk1.dependency-license-report")
}

group = "io.github.sbradl"

kotlin {
    jvmToolchain(21)
    jvm()
}

dependencyLocking {
    lockAllConfigurations()
}

licenseReport {
    configurations = arrayOf("jvmRuntimeClasspath")
    allowedLicensesFile = rootProject.file("gradle/allowed-licenses.json")
    renderers = arrayOf<ReportRenderer>(TextReportRenderer())
    filters =
        arrayOf(
            com.github.jk1.license.filter
                .LicenseBundleNormalizer(),
        )
}
