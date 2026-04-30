import 'package:freezed_annotation/freezed_annotation.dart';

part 'translation_rule.freezed.dart';
part 'translation_rule.g.dart';

@freezed
sealed class TranslationRule with _$TranslationRule {
  const factory TranslationRule({
    required String id,
    required String providerId,
    required String roleName,
    required String systemPrompt,
    @Default('{}') String customRules,
    @Default(false) bool isDefault,
  }) = _TranslationRule;

  factory TranslationRule.fromJson(Map<String, dynamic> json) =>
      _$TranslationRuleFromJson(json);
}
