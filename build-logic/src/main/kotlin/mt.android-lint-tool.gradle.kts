val libs = versionCatalogs.named("libs")

val patchedLintToolDependencies =
    mapOf(
        "bouncycastle-bcprov" to "AGP lint pulls Bouncy Castle 1.80.2, which has CVE-2026-8763 and CVE-2026-13506",
        "bouncycastle-bcpkix" to "AGP lint pulls Bouncy Castle 1.80.2, which has CVE-2026-5588",
        "bouncycastle-bcutil" to "keeps Bouncy Castle at one version",
        "guava" to "AGP lint pulls Guava 33.4.0, which has CVE-2026-102554",
        "commons-lang3" to "AGP lint pulls commons-lang3 3.16.0, which has CVE-2025-48924",
        "httpclient" to "AGP lint pulls httpclient 4.5.6, which has CVE-2020-13956",
    )

dependencies {
    constraints {
        patchedLintToolDependencies.forEach { (library, reason) ->
            add("androidLintTool", libs.findLibrary(library).get()) { because(reason) }
        }
    }
}
