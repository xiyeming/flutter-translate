import 'package:freezed_annotation/freezed_annotation.dart';

part 'translation_result.freezed.dart';
part 'translation_result.g.dart';

@freezed
sealed class TranslationResult with _$TranslationResult {
  const factory TranslationResult({
    required String providerId,
    required String providerName,
    required String sourceText,
    required String translatedText,
    required int responseTimeMs,
    @Default(true) bool isSuccess,
    String? errorMessage,
    @Default(0) int promptTokens,
    @Default(0) int completionTokens,
    @Default(0) int totalTokens,
  }) = _TranslationResult;

  factory TranslationResult.fromJson(Map<String, dynamic> json) =>
      _$TranslationResultFromJson(json);
}
