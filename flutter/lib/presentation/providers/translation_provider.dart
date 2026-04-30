import 'package:flutter_riverpod/flutter_riverpod.dart';

final translationProvider = StateNotifierProvider<TranslationNotifier, TranslationState>((ref) {
  return TranslationNotifier();
});

class TranslationNotifier extends StateNotifier<TranslationState> {
  TranslationNotifier() : super(const TranslationState());

  Future<void> translate(String text) async {
    // TODO: Implement translation logic
  }

  Future<void> compare(String text) async {
    // TODO: Implement comparison logic
  }
}

class TranslationState {
  final String text;
  final bool isLoading;
  final String? error;

  const TranslationState({
    this.text = '',
    this.isLoading = false,
    this.error,
  });
}
