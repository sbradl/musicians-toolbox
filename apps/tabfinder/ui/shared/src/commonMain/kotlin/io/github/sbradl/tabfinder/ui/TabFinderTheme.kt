package io.github.sbradl.tabfinder.ui

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable

@Composable
fun TabFinderTheme(
    prefersDark: Boolean,
    content: @Composable () -> Unit,
) {
    val colorScheme = if (prefersDark) darkColorScheme() else lightColorScheme()
    MaterialTheme(colorScheme = colorScheme) {
        Surface(content = content)
    }
}
