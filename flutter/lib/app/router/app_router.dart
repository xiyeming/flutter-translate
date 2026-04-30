import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';
import '../../presentation/pages/floating_page.dart';
import '../../presentation/pages/main_page.dart';
import '../../presentation/pages/compare_page.dart';
import '../../presentation/pages/settings_page.dart';
import '../../presentation/pages/provider_edit_page.dart';

class RouteNames {
  static const floating = '/';
  static const main = '/main';
  static const compare = '/compare';
  static const settings = '/settings';
  static const providerEdit = '/settings/provider';
}

final appRouter = GoRouter(
  initialLocation: RouteNames.floating,
  debugLogDiagnostics: true,
  routes: [
    GoRoute(
      path: RouteNames.floating,
      name: RouteNames.floating,
      builder: (context, state) => const FloatingPage(),
    ),
    GoRoute(
      path: RouteNames.main,
      name: RouteNames.main,
      builder: (context, state) => const MainPage(),
    ),
    GoRoute(
      path: RouteNames.compare,
      name: RouteNames.compare,
      builder: (context, state) => const ComparePage(),
    ),
    GoRoute(
      path: RouteNames.settings,
      name: RouteNames.settings,
      builder: (context, state) => const SettingsPage(),
    ),
    GoRoute(
      path: RouteNames.providerEdit,
      name: RouteNames.providerEdit,
      builder: (context, state) {
        final providerId = state.uri.queryParameters['id'];
        final defaultName = state.uri.queryParameters['name'];
        final defaultModel = state.uri.queryParameters['model'];
        return ProviderEditPage(
          providerId: providerId,
          defaultName: defaultName,
          defaultModel: defaultModel,
        );
      },
    ),
  ],
  errorBuilder: (context, state) => Scaffold(
    body: Center(
      child: Text('页面未找到: ${state.error}'),
    ),
  ),
);