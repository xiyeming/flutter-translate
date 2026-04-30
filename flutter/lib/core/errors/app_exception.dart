sealed class AppException implements Exception {
  final String message;
  final String? code;

  const AppException(this.message, {this.code});

  @override
  String toString() => code != null ? '$code: $message' : message;
}

class TranslationException extends AppException {
  const TranslationException(super.message, {super.code});
}

class ConfigException extends AppException {
  const ConfigException(super.message, {super.code});
}

class SessionException extends AppException {
  const SessionException(super.message, {super.code});
}

class OcrException extends AppException {
  const OcrException(super.message, {super.code});
}

class ShortcutException extends AppException {
  const ShortcutException(super.message, {super.code});
}

class SystemException extends AppException {
  const SystemException(super.message, {super.code});
}
