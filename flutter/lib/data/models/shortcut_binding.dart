import 'package:freezed_annotation/freezed_annotation.dart';

part 'shortcut_binding.freezed.dart';
part 'shortcut_binding.g.dart';

@freezed
sealed class ShortcutBinding with _$ShortcutBinding {
  const factory ShortcutBinding({
    required String id,
    required String action,
    required String keyCombination,
    @Default(true) bool enabled,
  }) = _ShortcutBinding;

  factory ShortcutBinding.fromJson(Map<String, dynamic> json) =>
      _$ShortcutBindingFromJson(json);
}
