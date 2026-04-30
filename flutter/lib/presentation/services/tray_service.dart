import 'dart:async';

class TrayService {
  static final TrayService _instance = TrayService._internal();
  factory TrayService() => _instance;
  TrayService._internal();

  final _actionController = StreamController<String>.broadcast();
  Stream<String> get actionStream => _actionController.stream;

  Future<void> initialize() async {
    // TODO: Initialize system tray via FFI
  }

  Future<void> updateTooltip(String tooltip) async {
    // TODO: Update tray tooltip
  }

  void startListening() {
    // TODO: Start listening for tray actions
  }

  void dispose() {
    _actionController.close();
  }
}
