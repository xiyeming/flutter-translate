import '../../data/datasources/ffi_datasource.dart';
import '../../data/models/provider_config.dart';

abstract class ConfigRepository {
  Future<List<ProviderConfig>> getProviders();
  Future<void> saveProvider(ProviderConfig config);
  Future<void> deleteProvider(String id);
}

class ConfigRepositoryImpl implements ConfigRepository {
  final FfiDatasource _datasource;

  ConfigRepositoryImpl(this._datasource);

  @override
  Future<List<ProviderConfig>> getProviders() {
    return _datasource.getProviders();
  }

  @override
  Future<void> saveProvider(ProviderConfig config) {
    return _datasource.saveProvider(config);
  }

  @override
  Future<void> deleteProvider(String id) {
    return _datasource.deleteProvider(id);
  }
}
