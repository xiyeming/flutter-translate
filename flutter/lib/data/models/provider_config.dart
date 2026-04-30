import 'package:freezed_annotation/freezed_annotation.dart';

part 'provider_config.freezed.dart';
part 'provider_config.g.dart';

@freezed
sealed class ProviderConfig with _$ProviderConfig {
  const factory ProviderConfig({
    required String id,
    required String name,
    String? apiKey,
    String? apiUrl,
    required String model,
    @Default('api_key') String authType,
    @Default(true) bool isActive,
    @Default(0) int sortOrder,
    String? systemPrompt,
    required DateTime createdAt,
  }) = _ProviderConfig;

  factory ProviderConfig.fromJson(Map<String, dynamic> json) =>
      _$ProviderConfigFromJson(json);
}
