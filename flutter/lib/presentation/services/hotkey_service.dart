import 'dart:async';
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
  final _ffi = FfiDatasource();

  Future<void> registerAll() async {
    try {
      final bindings = await _ffi.getShortcuts();
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
        await _ffi.registerHotkeys(defaults);
      } else {
        await _ffi.registerHotkeys(bindings);
      }

      _startPolling();
    } catch (e) {
      // Hotkey registration failed — non-fatal
    }
  }

  Future<void> updateAndReregister(List<ShortcutBinding> bindings) async {
    try {
      _stopPolling();
      await _ffi.unregisterHotkeys();
      for (final b in bindings) {
        await _ffi.updateShortcut(b);
      }
      await _ffi.registerHotkeys(bindings.where((b) => b.enabled).toList());
      _startPolling();
    } catch (_) {}
  }

  void _startPolling() {
    _pollTimer?.cancel();
    _pollTimer = Timer.periodic(const Duration(milliseconds: 200), (_) async {
      try {
        final event = await _ffi.pollHotkeyEvent();
        if (event != null && event.isNotEmpty) {
          _hotkeyController.add(event);
        }
      } catch (_) {}
    });
  }

  void _stopPolling() {
    _pollTimer?.cancel();
    _pollTimer = null;
  }

  void dispose() {
    _stopPolling();
    _hotkeyController.close();
  }
}
