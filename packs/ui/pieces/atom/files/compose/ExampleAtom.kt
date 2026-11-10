package com.example.app.ui.atoms

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ProvideTextStyle
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
// @tfverse if !stack.compose-multiplatform
import androidx.compose.ui.tooling.preview.Preview
// @tfverse end
import androidx.compose.ui.unit.dp

/** Visual emphasis of an [ExampleAtom]. */
enum class ExampleAtomTone { Neutral, Accent }

/** The smallest UI building block: a label-sized piece of content. */
@Composable
fun ExampleAtom(
    modifier: Modifier = Modifier,
    tone: ExampleAtomTone = ExampleAtomTone.Neutral,
    content: @Composable () -> Unit,
) {
    val colors = MaterialTheme.colorScheme
    Surface(
        modifier = modifier,
        shape = CircleShape,
        color = if (tone == ExampleAtomTone.Accent) colors.primary else colors.surfaceVariant,
        contentColor = if (tone == ExampleAtomTone.Accent) colors.onPrimary else colors.onSurfaceVariant,
    ) {
        ProvideTextStyle(MaterialTheme.typography.labelMedium) {
            Box(Modifier.padding(horizontal = 8.dp, vertical = 2.dp)) { content() }
        }
    }
}
// @tfverse if !stack.compose-multiplatform

@Preview
@Composable
private fun ExampleAtomPreview() {
    MaterialTheme { ExampleAtom(tone = ExampleAtomTone.Accent) { Text("New") } }
}
// @tfverse end
