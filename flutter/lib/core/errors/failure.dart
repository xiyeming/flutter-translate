sealed class Failure {
  final String message;

  const Failure(this.message);

  @override
  String toString() => message;
}

class TranslationFailure extends Failure {
  const TranslationFailure(super.message);
}

class NetworkFailure extends Failure {
  const NetworkFailure(super.message);
}

class ConfigFailure extends Failure {
  const ConfigFailure(super.message);
}
