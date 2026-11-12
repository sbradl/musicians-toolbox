import com.github.jk1.license.render.ReportRenderer
import com.github.jk1.license.render.TextReportRenderer
import org.gradle.api.tasks.testing.logging.TestExceptionFormat
import org.gradle.api.tasks.testing.logging.TestLogEvent

plugins {
    id("org.jetbrains.kotlin.multiplatform")
    id("com.github.jk1.dependency-license-report")
}

group = "io.github.sbradl"

kotlin {
    jvmToolchain(21)
    jvm()
}

tasks.withType<Test>().configureEach {
    testLogging.quiet {
        events(TestLogEvent.FAILED)
        exceptionFormat = TestExceptionFormat.FULL
        showStackTraces = false
    }
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
