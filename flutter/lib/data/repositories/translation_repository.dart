import '../../data/datasources/ffi_datasource.dart';
import '../../data/models/translation_result.dart';
import 'package:flutter_translate/src/rust/ffi/bridge.dart' as bridge;

abstract class TranslationRepository {
  Future<TranslationResult> translate({
    required String text,
    required String sourceLang,
    required String targetLang,
    required String providerId,
  });

  Future<List<TranslationResult>> compare({
    required String text,
    required String sourceLang,
    required String targetLang,
    required List<String> providerIds,
  });

  Future<String> detectLanguage(String text);
  Future<bridge.TestResult> testProvider(String providerId);
}

class TranslationRepositoryImpl implements TranslationRepository {
  final FfiDatasource _datasource;

  TranslationRepositoryImpl(this._datasource);

  @override
  Future<TranslationResult> translate({
    required String text,
    required String sourceLang,
    required String targetLang,
    required String providerId,
  }) {
    return _datasource.translate(
      text: text,
      sourceLang: sourceLang,
      targetLang: targetLang,
      providerId: providerId,
    );
  }

  @override
  Future<List<TranslationResult>> compare({
    required String text,
    required String sourceLang,
    required String targetLang,
    required List<String> providerIds,
  }) {
    return _datasource.translateCompare(
      text: text,
      sourceLang: sourceLang,
      targetLang: targetLang,
      providerIds: providerIds,
    );
  }

  @override
  Future<String> detectLanguage(String text) {
    return _datasource.detectLanguage(text);
  }

  @override
  Future<bridge.TestResult> testProvider(String providerId) {
    return _datasource.testProvider(providerId);
  }
}
