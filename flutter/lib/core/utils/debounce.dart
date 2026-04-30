import 'dart:async';

class Debounce {
  Timer? _timer;

  void call(void Function() action, Duration duration) {
    _timer?.cancel();
    _timer = Timer(duration, action);
  }

  void dispose() {
    _timer?.cancel();
  }
}
