import 'dart:async';

class ClipboardService {
  static final ClipboardService _instance = ClipboardService._internal();
  factory ClipboardService() => _instance;
  ClipboardService._internal();

  final _contentController = StreamController<String>.broadcast();
  Stream<String> get contentStream => _contentController.stream;

  Future<String?> getText() async {
    // TODO: Implement via FFI
    return null;
  }

  Future<void> setText(String text) async {
    // TODO: Implement via FFI
  }

  void startListening() {
    // TODO: Start clipboard monitoring via FFI
  }

  void stopListening() {
    // TODO: Stop clipboard monitoring
  }

  void dispose() {
    _contentController.close();
  }
}
