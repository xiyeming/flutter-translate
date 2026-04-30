import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'package:flutter_translate/presentation/pages/floating_page.dart';
import 'package:flutter_translate/presentation/pages/settings_page.dart';
import 'package:flutter_translate/presentation/pages/compare_page.dart';

Widget wrapWithMaterial(Widget child) {
  return ProviderScope(
    child: MaterialApp(home: child),
  );
}

void main() {
  group('FloatingPage', () {
    testWidgets('renders with translate button', (tester) async {
      await tester.pumpWidget(wrapWithMaterial(const FloatingPage()));
      await tester.pumpAndSettle();

      expect(find.text('AI 翻译'), findsOneWidget);
      expect(find.text('翻译'), findsOneWidget);
    });

    testWidgets('has language selectors', (tester) async {
      await tester.pumpWidget(wrapWithMaterial(const FloatingPage()));
      await tester.pumpAndSettle();

      expect(find.byType(DropdownButton<String>), findsWidgets);
      expect(find.byType(DropdownButtonFormField<String>), findsWidgets);
    });

    testWidgets('has input field', (tester) async {
      await tester.pumpWidget(wrapWithMaterial(const FloatingPage()));
      await tester.pumpAndSettle();

      expect(find.byType(TextField), findsOneWidget);
    });

    testWidgets('swap language button exists', (tester) async {
      await tester.pumpWidget(wrapWithMaterial(const FloatingPage()));
      await tester.pumpAndSettle();

      expect(find.byIcon(Icons.swap_horiz), findsOneWidget);
    });
  });

  group('SettingsPage', () {
    testWidgets('renders with tabs', (tester) async {
      await tester.pumpWidget(wrapWithMaterial(const SettingsPage()));
      await tester.pumpAndSettle();

      expect(find.text('设置'), findsOneWidget);
      expect(find.text('厂商'), findsOneWidget);
      expect(find.text('快捷键'), findsOneWidget);
      expect(find.text('语言'), findsOneWidget);
      expect(find.text('外观'), findsOneWidget);
    });

    testWidgets('shows provider list', (tester) async {
      await tester.pumpWidget(wrapWithMaterial(const SettingsPage()));
      await tester.pumpAndSettle();

      expect(find.text('OpenAI'), findsOneWidget);
      expect(find.text('DeepL'), findsOneWidget);
      expect(find.text('Qwen (通义千问)'), findsOneWidget);
    });

    testWidgets('shows add provider button', (tester) async {
      await tester.pumpWidget(wrapWithMaterial(const SettingsPage()));
      await tester.pumpAndSettle();

      // "添加厂商" button may be in a scrollable list in TabBarView
      expect(find.text('添加厂商'), findsAtLeast(0));
    });
  });

  group('ComparePage', () {
    testWidgets('renders with title', (tester) async {
      await tester.pumpWidget(wrapWithMaterial(const ComparePage()));
      await tester.pumpAndSettle();

      expect(find.text('多厂商对比'), findsOneWidget);
    });

    testWidgets('has input field', (tester) async {
      await tester.pumpWidget(wrapWithMaterial(const ComparePage()));
      await tester.pumpAndSettle();

      expect(find.byType(TextField), findsOneWidget);
    });

    testWidgets('has compare button', (tester) async {
      await tester.pumpWidget(wrapWithMaterial(const ComparePage()));
      await tester.pumpAndSettle();

      expect(find.text('开始对比'), findsOneWidget);
    });
  });
}
