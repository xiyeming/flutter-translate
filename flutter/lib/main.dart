import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:window_manager/window_manager.dart';
import 'package:tray_manager/tray_manager.dart';
import 'package:flutter_translate/src/rust/frb_generated.dart';
import 'package:flutter_translate/src/rust/ffi/bridge.dart' as bridge;
import 'presentation/services/hotkey_service.dart';
import 'app/router/app_router.dart';
import 'app/app.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  await RustLib.init();

  try {
    await bridge.initServices();
  } catch (e) {
    debugPrint('Service init failed (non-fatal): $e');
  }

  // Start hotkey listener
  try {
    HotkeyService().registerAll();
  } catch (e) {
    debugPrint('Hotkey init failed (non-fatal): $e');
  }

  await windowManager.ensureInitialized();

  await windowManager.setPreventClose(true);

  const windowOptions = WindowOptions(
    size: Size(400, 600),
    minimumSize: Size(350, 400),
    center: true,
    backgroundColor: Colors.transparent,
    titleBarStyle: TitleBarStyle.hidden,
    windowButtonVisibility: false,
    alwaysOnTop: true,
    skipTaskbar: false,
  );

  windowManager.waitUntilReadyToShow(windowOptions, () async {
    await windowManager.show();
    await windowManager.focus();
  });

  await _initTray();

  runApp(
    const ProviderScope(
      child: FlutterTranslateApp(),
    ),
  );
}

Future<void> _initTray() async {
  try {
    await trayManager.setIcon(
      'assets/icons/tray_icon.png',
    );

    final menu = Menu(items: [
      MenuItem(key: 'show', label: '显示窗口'),
      MenuItem(key: 'ocr', label: '截图翻译'),
      MenuItem.separator(),
      MenuItem(key: 'settings', label: '设置'),
      MenuItem.separator(),
      MenuItem(key: 'quit', label: '退出'),
    ]);

    await trayManager.setContextMenu(menu);
  } catch (e) {
    debugPrint('Tray init failed (non-fatal): $e');
  }
}

class TrayApp extends ConsumerStatefulWidget {
  final Widget child;

  const TrayApp({super.key, required this.child});

  @override
  ConsumerState<TrayApp> createState() => _TrayAppState();
}

class _TrayAppState extends ConsumerState<TrayApp> with TrayListener {
  @override
  void initState() {
    super.initState();
    trayManager.addListener(this);
  }

  @override
  void dispose() {
    trayManager.removeListener(this);
    super.dispose();
  }

  @override
  void onTrayIconMouseDown() {
    windowManager.show();
    windowManager.focus();
  }

  @override
  void onTrayIconRightMouseDown() {}

  @override
  void onTrayMenuItemClick(MenuItem menuItem) async {
    switch (menuItem.key) {
      case 'show':
        await windowManager.show();
        await windowManager.focus();
      case 'ocr':
        await windowManager.show();
        await windowManager.focus();
        HotkeyService().simulateEvent('ocr_screenshot');
      case 'settings':
        await windowManager.show();
        await windowManager.focus();
        appRouter.go('/settings');
      case 'quit':
        await windowManager.setPreventClose(false);
        await windowManager.close();
    }
  }

  @override
  Widget build(BuildContext context) {
    return widget.child;
  }
}