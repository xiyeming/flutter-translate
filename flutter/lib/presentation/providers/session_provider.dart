import 'package:flutter_riverpod/flutter_riverpod.dart';

final sessionProvider = StateNotifierProvider<SessionNotifier, SessionState>((ref) {
  return SessionNotifier();
});

class SessionNotifier extends StateNotifier<SessionState> {
  SessionNotifier() : super(const SessionState());

  Future<void> startSession() async {
    // TODO: Start translation session
  }

  Future<void> endSession() async {
    // TODO: End translation session
  }
}

class SessionState {
  final bool isActive;
  final DateTime? startTime;

  const SessionState({
    this.isActive = false,
    this.startTime,
  });
}
