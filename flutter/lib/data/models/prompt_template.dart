import 'package:freezed_annotation/freezed_annotation.dart';

part 'prompt_template.freezed.dart';
part 'prompt_template.g.dart';

@freezed
sealed class PromptTemplate with _$PromptTemplate {
  const factory PromptTemplate({
    required String id,
    required String name,
    required String content,
    @Default(false) bool isActive,
    required DateTime createdAt,
  }) = _PromptTemplate;

  factory PromptTemplate.fromJson(Map<String, dynamic> json) =>
      _$PromptTemplateFromJson(json);
}
