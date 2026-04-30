import '../../data/datasources/ffi_datasource.dart';
import '../../data/models/active_session.dart';

abstract class SessionRepository {
  Future<ActiveSession> getActiveSession();
  Future<void> updateSession({String? providerId, List<String>? compareProviders});
}

class SessionRepositoryImpl implements SessionRepository {
  final FfiDatasource _datasource;

  SessionRepositoryImpl(this._datasource);

  @override
  Future<ActiveSession> getActiveSession() {
    return _datasource.getActiveSession();
  }

  @override
  Future<void> updateSession({String? providerId, List<String>? compareProviders}) {
    return _datasource.updateSession(providerId: providerId, compareProviders: compareProviders);
  }
}
