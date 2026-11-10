import 'package:flutter/material.dart';

/// A distinct, labelled region of the screen, built from molecules.
class ExampleOrganism extends StatelessWidget {
  const ExampleOrganism({super.key, required this.heading, required this.child, this.actions = const []});

  /// Heading that names the section.
  final String heading;

  /// Controls shown next to the heading.
  final List<Widget> actions;

  /// Section content.
  final Widget child;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Card.outlined(
      margin: EdgeInsets.zero,
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          spacing: 16,
          children: [
            Row(
              children: [
                Expanded(
                  child: Semantics(header: true, child: Text(heading, style: theme.textTheme.titleLarge)),
                ),
                ...actions,
              ],
            ),
            child,
          ],
        ),
      ),
    );
  }
}
