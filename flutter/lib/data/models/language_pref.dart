import 'package:freezed_annotation/freezed_annotation.dart';

part 'language_pref.freezed.dart';
part 'language_pref.g.dart';

@freezed
sealed class LanguagePref with _$LanguagePref {
  const factory LanguagePref({
    required String code,
    required String displayName,
    @Default(0) int usageCount,
    @Default(false) bool isFavorite,
  }) = _LanguagePref;

  factory LanguagePref.fromJson(Map<String, dynamic> json) =>
      _$LanguagePrefFromJson(json);
}
