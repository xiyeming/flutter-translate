import 'package:freezed_annotation/freezed_annotation.dart';

part 'active_session.freezed.dart';
part 'active_session.g.dart';

@freezed
sealed class ActiveSession with _$ActiveSession {
  const factory ActiveSession({
    @Default('') String lastProviderId,
    @Default([]) List<String> lastCompareProviders,
    required DateTime lastUsed,
  }) = _ActiveSession;

  factory ActiveSession.fromJson(Map<String, dynamic> json) =>
      _$ActiveSessionFromJson(json);
}
