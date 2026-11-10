import 'package:flutter/material.dart';

/// Groups atoms under one accessible label, such as a field with its hint.
class ExampleMolecule extends StatelessWidget {
  const ExampleMolecule({super.key, required this.label, this.description, this.children = const []});

  /// Names the group, also for assistive technology.
  final String label;

  /// Extra guidance shown under the label.
  final String? description;

  /// Content, usually atoms.
  final List<Widget> children;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final description = this.description;

    return Semantics(
      container: true,
      label: label,
      hint: description,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        spacing: 8,
        children: [
          ExcludeSemantics(child: Text(label, style: theme.textTheme.titleSmall)),
          if (description != null)
            ExcludeSemantics(
              child: Text(
                description,
                style: theme.textTheme.bodySmall?.copyWith(color: theme.colorScheme.onSurfaceVariant),
              ),
            ),
          Wrap(spacing: 8, runSpacing: 8, crossAxisAlignment: WrapCrossAlignment.center, children: children),
        ],
      ),
    );
  }
}
