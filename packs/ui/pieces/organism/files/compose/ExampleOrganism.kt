package com.example.app.ui.organisms

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedCard
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
// @tfverse if !stack.compose-multiplatform
import androidx.compose.ui.tooling.preview.Preview
// @tfverse end
import androidx.compose.ui.unit.dp

/** A distinct, labelled region of the screen, built from molecules. */
@Composable
fun ExampleOrganism(
    heading: String,
    modifier: Modifier = Modifier,
    actions: @Composable RowScope.() -> Unit = {},
    content: @Composable () -> Unit,
) {
    OutlinedCard(modifier = modifier) {
        Column(Modifier.padding(24.dp), verticalArrangement = Arrangement.spacedBy(16.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Text(
                    heading,
                    modifier = Modifier.weight(1f).semantics { heading() },
                    style = MaterialTheme.typography.titleLarge,
                )
                actions()
            }
            content()
        }
    }
}
// @tfverse if !stack.compose-multiplatform

@Preview
@Composable
private fun ExampleOrganismPreview() {
    MaterialTheme {
        ExampleOrganism(heading = "Team", actions = { Button(onClick = {}) { Text("Invite") } }) {
            Text("Three members")
        }
    }
}
// @tfverse end
