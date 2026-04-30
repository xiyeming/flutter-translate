import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:window_manager/window_manager.dart';
import 'package:flutter_translate/main.dart';
import 'router/app_router.dart';
import 'theme/app_theme.dart';

class FlutterTranslateApp extends ConsumerStatefulWidget {
  const FlutterTranslateApp({super.key});

  @override
  ConsumerState<FlutterTranslateApp> createState() => _FlutterTranslateAppState();
}

class _FlutterTranslateAppState extends ConsumerState<FlutterTranslateApp> with WindowListener {
  @override
  void initState() {
    super.initState();
    windowManager.addListener(this);
  }

  @override
  void dispose() {
    windowManager.removeListener(this);
    super.dispose();
  }

  @override
  void onWindowClose() async {
    bool isPreventClose = await windowManager.isPreventClose();
    if (isPreventClose) {
      await windowManager.hide();
    }
  }

  @override
  Widget build(BuildContext context) {
    return TrayApp(
      child: MaterialApp.router(
        title: 'Flutter Translate',
        debugShowCheckedModeBanner: false,
        theme: AppTheme.light,
        darkTheme: AppTheme.dark,
        themeMode: ThemeMode.system,
        routerConfig: appRouter,
      ),
    );
  }
}