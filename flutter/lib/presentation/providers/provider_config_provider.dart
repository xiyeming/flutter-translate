import 'package:flutter_riverpod/flutter_riverpod.dart';

final providerConfigProvider = StateNotifierProvider<ProviderConfigNotifier, ProviderConfigState>((ref) {
  return ProviderConfigNotifier();
});

class ProviderConfigNotifier extends StateNotifier<ProviderConfigState> {
  ProviderConfigNotifier() : super(const ProviderConfigState());

  Future<void> loadProviders() async {
    // TODO: Load provider configurations
  }

  Future<void> addProvider(Map<String, dynamic> config) async {
    // TODO: Add new provider configuration
  }

  Future<void> updateProvider(String id, Map<String, dynamic> config) async {
    // TODO: Update provider configuration
  }

  Future<void> deleteProvider(String id) async {
    // TODO: Delete provider configuration
  }
}

class ProviderConfigState {
  final List<Map<String, dynamic>> providers;
  final String? selectedId;

  const ProviderConfigState({
    this.providers = const [],
    this.selectedId,
  });
}
