import 'package:example_app/ui/organisms/example_organism.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  testWidgets('shows its heading, actions and content', (tester) async {
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: ExampleOrganism(
            heading: 'Team',
            actions: [FilledButton(onPressed: () {}, child: const Text('Invite'))],
            child: const Text('Three members'),
          ),
        ),
      ),
    );

    expect(find.text('Team'), findsOneWidget);
    expect(find.text('Invite'), findsOneWidget);
    expect(find.text('Three members'), findsOneWidget);
  });
}
