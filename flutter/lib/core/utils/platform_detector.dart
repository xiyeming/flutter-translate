import 'dart:io';

class PlatformDetector {
  static bool get isLinux => Platform.isLinux;
  static bool get isWayland => Platform.environment['WAYLAND_DISPLAY'] != null;

  static String? get currentDesktop {
    return Platform.environment['XDG_CURRENT_DESKTOP'];
  }

  static bool get isKde => currentDesktop?.toLowerCase().contains('kde') ?? false;
  static bool get isHyprland => currentDesktop?.toLowerCase().contains('hyprland') ?? false;
  static bool get isGnome => currentDesktop?.toLowerCase().contains('gnome') ?? false;
}
