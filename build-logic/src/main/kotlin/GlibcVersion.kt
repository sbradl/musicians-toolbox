import java.io.Serializable

data class GlibcVersion(
    val major: Int,
    val minor: Int,
) : Comparable<GlibcVersion>,
    Serializable {
    override fun compareTo(other: GlibcVersion) = compareValuesBy(this, other, GlibcVersion::major, GlibcVersion::minor)

    override fun toString() = "$major.$minor"
}
