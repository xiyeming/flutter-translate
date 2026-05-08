import 'dart:async';
import 'package:flutter/foundation.dart';
import '../../data/datasources/ffi_datasource.dart';
import '../../data/models/shortcut_binding.dart';

class HotkeyService {
  static final HotkeyService _instance = HotkeyService._internal();
  factory HotkeyService() => _instance;
  HotkeyService._internal();

  final _hotkeyController = StreamController<String>.broadcast();
  Stream<String> get hotkeyStream => _hotkeyController.stream;

  /// 模拟热键事件（用于托盘菜单等非键盘触发场景）
  void simulateEvent(String action) {
    _hotkeyController.add(action);
  }

  Timer? _pollTimer;
  Timer? _watchdogTimer;
  DateTime? _lastWatchdogTick;
  final _ffi = FfiDatasource();

  Future<void> registerAll() async {
    try {
      final bindings = await _ffi.getShortcuts();
      debugPrint('[hotkey] registerAll: found ${bindings.length} bindings');
      if (bindings.isEmpty) {
        // Insert default shortcuts if none exist
        final defaults = [
          ShortcutBinding(id: 'translate_selected', action: 'translate_selected', keyCombination: 'Super+Alt+F', enabled: true),
          ShortcutBinding(id: 'ocr_screenshot', action: 'ocr_screenshot', keyCombination: 'Ctrl+Shift+S', enabled: true),
          ShortcutBinding(id: 'toggle_window', action: 'toggle_window', keyCombination: 'Ctrl+Shift+F', enabled: true),
        ];
        for (final b in defaults) {
          await _ffi.updateShortcut(b);
        }
        debugPrint('[hotkey] registering ${defaults.length} default shortcuts');
        await _ffi.registerHotkeys(defaults);
      } else {
        debugPrint('[hotkey] registering ${bindings.length} saved shortcuts');
        await _ffi.registerHotkeys(bindings);
      }

      _startPolling();
    } catch (e) {
      debugPrint('[hotkey] registerAll failed: $e');
    }
  }

  Future<void> updateAndReregister(List<ShortcutBinding> bindings) async {
    try {
      debugPrint('[hotkey] updateAndReregister: ${bindings.length} bindings');
      _stopPolling();
      await _ffi.unregisterHotkeys();
      debugPrint('[hotkey] unregisterHotkeys done');
      for (final b in bindings) {
        await _ffi.updateShortcut(b);
      }
      final enabled = bindings.where((b) => b.enabled).toList();
      debugPrint('[hotkey] registering ${enabled.length} enabled shortcuts');
      await _ffi.registerHotkeys(enabled);
      _startPolling();
    } catch (e) {
      debugPrint('[hotkey] updateAndReregister failed: $e');
    }
  }

  void _startPolling() {
    _pollTimer?.cancel();
    debugPrint('[hotkey] polling started (200ms interval)');
    _pollTimer = Timer.periodic(const Duration(milliseconds: 200), (_) async {
      try {
        final event = await _ffi.pollHotkeyEvent();
        if (event != null && event.isNotEmpty) {
          debugPrint('[hotkey] polled event: $event');
          _hotkeyController.add(event);
        }
      } catch (e) {
        debugPrint('[hotkey] poll error: $e');
      }
    });
    _startWatchdog();
  }

  void _startWatchdog() {
    _watchdogTimer?.cancel();
    _lastWatchdogTick = DateTime.now();
    _watchdogTimer = Timer.periodic(const Duration(minutes: 1), (_) async {
      final now = DateTime.now();
      final last = _lastWatchdogTick;
      _lastWatchdogTick = now;
      if (last == null) return;

      final diff = now.difference(last);
      // 如果两次 watchdog 间隔超过 3 分钟，说明系统可能从睡眠/锁屏中恢复
      if (diff.inMinutes > 3) {
        debugPrint('[hotkey] System resumed from sleep/lock (gap ${diff.inMinutes}m), re-registering hotkeys');
        await _reRegister();
      }
    });
  }

  Future<void> _reRegister() async {
    try {
      final bindings = await _ffi.getShortcuts();
      final enabled = bindings.where((b) => b.enabled).toList();
      if (enabled.isEmpty) return;
      await _ffi.unregisterHotkeys();
      await _ffi.registerHotkeys(enabled);
      debugPrint('[hotkey] Re-registered ${enabled.length} hotkeys after resume');
    } catch (e) {
      debugPrint('[hotkey] Re-register failed: $e');
    }
  }

  void _stopPolling() {
    _pollTimer?.cancel();
    _pollTimer = null;
    _watchdogTimer?.cancel();
    _watchdogTimer = null;
  }

  void dispose() {
    _stopPolling();
    _hotkeyController.close();
  }
}
