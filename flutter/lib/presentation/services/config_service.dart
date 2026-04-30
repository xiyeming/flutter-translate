class ConfigService {
  static final ConfigService _instance = ConfigService._internal();
  factory ConfigService() => _instance;
  ConfigService._internal();

  Future<T?> get<T>(String key) async {
    // TODO: Get config value via FFI
    return null;
  }

  Future<void> set<T>(String key, T value) async {
    // TODO: Set config value via FFI
  }

  Future<void> load() async {
    // TODO: Load all config via FFI
  }

  Future<void> save() async {
    // TODO: Save all config via FFI
  }
}
