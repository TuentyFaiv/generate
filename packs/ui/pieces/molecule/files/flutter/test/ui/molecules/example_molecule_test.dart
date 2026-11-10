import 'package:example_app/ui/molecules/example_molecule.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  testWidgets('labels the group for assistive technology', (tester) async {
    final semantics = tester.ensureSemantics();
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: ExampleMolecule(
            label: 'Notifications',
            description: 'Choose how we reach you.',
            children: [TextButton(onPressed: () {}, child: const Text('Email'))],
          ),
        ),
      ),
    );

    expect(find.bySemanticsLabel('Notifications'), findsOneWidget);
    expect(find.text('Email'), findsOneWidget);
    semantics.dispose();
  });

  testWidgets('leaves out the description when there is none', (tester) async {
    await tester.pumpWidget(const MaterialApp(home: Scaffold(body: ExampleMolecule(label: 'Notifications'))));

    expect(find.byType(Text), findsOneWidget);
  });
}
