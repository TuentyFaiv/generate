package com.example.app.ui.molecules

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.isTraversalGroup
import androidx.compose.ui.semantics.semantics
// @tfverse if !stack.compose-multiplatform
import androidx.compose.ui.tooling.preview.Preview
// @tfverse end
import androidx.compose.ui.unit.dp

/** Groups atoms under one accessible label, such as a field with its hint. */
@OptIn(ExperimentalLayoutApi::class)
@Composable
fun ExampleMolecule(
    label: String,
    modifier: Modifier = Modifier,
    description: String? = null,
    content: @Composable () -> Unit,
) {
    Column(
        modifier = modifier.semantics { isTraversalGroup = true },
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        Text(label, style = MaterialTheme.typography.titleSmall)
        if (description != null) {
            Text(
                description,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
        FlowRow(
            horizontalArrangement = Arrangement.spacedBy(8.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp),
            itemVerticalAlignment = Alignment.CenterVertically,
        ) { content() }
    }
}
// @tfverse if !stack.compose-multiplatform

@Preview
@Composable
private fun ExampleMoleculePreview() {
    MaterialTheme {
        ExampleMolecule(label = "Notifications", description = "Choose how we reach you.") {
            TextButton(onClick = {}) { Text("Email") }
        }
    }
}
// @tfverse end
