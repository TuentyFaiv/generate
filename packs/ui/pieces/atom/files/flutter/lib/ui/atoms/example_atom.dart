import 'package:flutter/material.dart';

/// Visual emphasis of an [ExampleAtom].
enum ExampleAtomTone { neutral, accent }

/// The smallest UI building block: a label-sized piece of content.
class ExampleAtom extends StatelessWidget {
  const ExampleAtom({super.key, required this.child, this.tone = ExampleAtomTone.neutral});

  /// Content, usually a [Text].
  final Widget child;

  /// Visual emphasis.
  final ExampleAtomTone tone;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final (background, foreground) = switch (tone) {
      ExampleAtomTone.neutral => (theme.colorScheme.surfaceContainerHighest, theme.colorScheme.onSurfaceVariant),
      ExampleAtomTone.accent => (theme.colorScheme.primary, theme.colorScheme.onPrimary),
    };

    return DecoratedBox(
      decoration: ShapeDecoration(color: background, shape: const StadiumBorder()),
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
        child: DefaultTextStyle.merge(
          style: theme.textTheme.labelMedium?.copyWith(color: foreground),
          child: child,
        ),
      ),
    );
  }
}
