import 'package:example_app/ui/atoms/example_atom.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  testWidgets('renders its child', (tester) async {
    await tester.pumpWidget(const MaterialApp(home: Scaffold(body: ExampleAtom(child: Text('New')))));

    expect(find.text('New'), findsOneWidget);
  });

  testWidgets('uses the primary color for the accent tone', (tester) async {
    await tester.pumpWidget(
      const MaterialApp(home: Scaffold(body: ExampleAtom(tone: ExampleAtomTone.accent, child: Text('New')))),
    );
    final context = tester.element(find.byType(ExampleAtom));
    final box = tester.widget<DecoratedBox>(find.descendant(of: find.byType(ExampleAtom), matching: find.byType(DecoratedBox)));

    expect((box.decoration as ShapeDecoration).color, Theme.of(context).colorScheme.primary);
  });
}
