import 'package:flutter_riverpod/flutter_riverpod.dart';

final settingsProvider = StateNotifierProvider<SettingsNotifier, SettingsState>((ref) {
  return SettingsNotifier();
});

class SettingsNotifier extends StateNotifier<SettingsState> {
  SettingsNotifier() : super(const SettingsState());

  Future<void> loadSettings() async {
    // TODO: Load settings from backend
  }

  Future<void> saveSettings(SettingsState settings) async {
    // TODO: Save settings to backend
  }
}

class SettingsState {
  final String theme;
  final String? lastProviderId;
  final bool autoDetect;

  const SettingsState({
    this.theme = 'system',
    this.lastProviderId,
    this.autoDetect = true,
  });
}
