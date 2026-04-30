class AppConstants {
  static const String appName = 'Flutter Translate';
  static const String appVersion = '0.1.0';

  // 响应时间阈值
  static const Duration fastResponse = Duration(milliseconds: 500);
  static const Duration normalResponse = Duration(milliseconds: 1500);
  static const Duration slowResponse = Duration(milliseconds: 3000);

  // 翻译限制
  static const int maxTranslateLength = 5000;
  static const int maxOcrImageSize = 10 * 1024 * 1024; // 10MB
  static const int maxCompareProviders = 4;

  // 窗口尺寸
  static const double floatingWindowWidth = 480;
  static const double floatingWindowMinHeight = 200;
  static const double floatingWindowMaxHeight = 600;

  // 默认快捷键
  static const String defaultTranslateShortcut = 'Ctrl+Alt+[';
  static const String defaultOcrShortcut = 'Ctrl+Alt+]';
  static const String defaultSwitchShortcut = 'Ctrl+Tab';
  static const String defaultCompareShortcut = 'Ctrl+Shift+M';
}
