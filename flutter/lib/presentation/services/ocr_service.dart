class OcrService {
  static final OcrService _instance = OcrService._internal();
  factory OcrService() => _instance;
  OcrService._internal();

  Future<String?> captureAndRecognize() async {
    // TODO: Implement screenshot capture and OCR via FFI
    return null;
  }
}
