class TranslationService {
  static final TranslationService _instance = TranslationService._internal();
  factory TranslationService() => _instance;
  TranslationService._internal();

  Future<String?> translate(String text, String providerId) async {
    // TODO: Implement translation via FFI
    return null;
  }

  Future<Map<String, String?>> compare(String text, List<String> providerIds) async {
    // TODO: Implement multi-provider comparison via FFI
    return {};
  }
}
