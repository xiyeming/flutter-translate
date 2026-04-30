import 'package:flutter_translate/src/rust/ffi/bridge.dart' as bridge;
import 'package:flutter_translate/src/rust/ffi/types.dart' as bridge_types;
import '../../core/errors/app_exception.dart';
import '../../data/models/provider_config.dart';
import '../../data/models/translation_result.dart';
import '../../data/models/active_session.dart';
import '../../data/models/shortcut_binding.dart';
import '../../data/models/prompt_template.dart';

class FfiDatasource {
  // ========== 翻译服务 ==========

  Future<TranslationResult> translate({
    required String text,
    required String sourceLang,
    required String targetLang,
    required String providerId,
    String? systemPromptOverride,
  }) async {
    try {
      final result = await bridge.translate(
        text: text,
        sourceLang: sourceLang,
        targetLang: targetLang,
        providerId: providerId,
        systemPromptOverride: systemPromptOverride,
      );
      return _mapTranslationResult(result);
    } catch (e) {
      throw TranslationException('翻译失败: $e');
    }
  }

  Future<List<TranslationResult>> translateCompare({
    required String text,
    required String sourceLang,
    required String targetLang,
    required List<String> providerIds,
    String? systemPromptOverride,
  }) async {
    try {
      final results = await bridge.translateCompare(
        text: text,
        sourceLang: sourceLang,
        targetLang: targetLang,
        providerIds: providerIds,
        systemPromptOverride: systemPromptOverride,
      );
      return results.map(_mapTranslationResult).toList();
    } catch (e) {
      throw TranslationException('对比翻译失败: $e');
    }
  }

  Future<String> detectLanguage(String text) async {
    try {
      return await bridge.detectLanguage(text: text);
    } catch (e) {
      throw TranslationException('语言检测失败: $e');
    }
  }

  // ========== 配置管理 ==========

  Future<List<ProviderConfig>> getProviders() async {
    try {
      final configs = await bridge.getProviders();
      return configs.map(_mapProviderConfig).toList();
    } catch (e) {
      throw ConfigException('获取厂商配置失败: $e');
    }
  }

  Future<void> saveProvider(ProviderConfig config) async {
    try {
      await bridge.saveProvider(config: _mapProviderConfigToBridge(config));
    } catch (e) {
      throw ConfigException('保存厂商配置失败: $e');
    }
  }

  Future<void> deleteProvider(String id) async {
    try {
      await bridge.deleteProvider(id: id);
    } catch (e) {
      throw ConfigException('删除厂商配置失败: $e');
    }
  }

  /// 测试厂商连接，返回 TestResult
  Future<bridge.TestResult> testProvider(String providerId) async {
    try {
      return await bridge.testProvider(providerId: providerId);
    } catch (e) {
      return bridge.TestResult(success: false, message: '测试连接异常: $e');
    }
  }

  Future<ActiveSession> getActiveSession() async {
    try {
      final session = await bridge.getActiveSession();
      return _mapActiveSession(session);
    } catch (e) {
      throw ConfigException('获取会话失败: $e');
    }
  }

  Future<void> updateSession({
    String? providerId,
    List<String>? compareProviders,
  }) async {
    try {
      await bridge.updateSession(
        providerId: providerId,
        compareProviders: compareProviders,
      );
    } catch (e) {
      throw ConfigException('更新会话失败: $e');
    }
  }

  // ========== 系统服务 ==========

  Future<String> ocrScreenshot() async {
    try {
      return await bridge.ocrScreenshot();
    } catch (e) {
      throw OcrException('OCR 截图识别失败: $e');
    }
  }

  Future<List<ShortcutBinding>> getShortcuts() async {
    try {
      final bindings = await bridge.getShortcuts();
      return bindings.map(_mapShortcutBinding).toList();
    } catch (e) {
      throw ShortcutException('获取快捷键配置失败: $e');
    }
  }

  Future<void> updateShortcut(ShortcutBinding binding) async {
    try {
      await bridge.updateShortcut(binding: _mapShortcutBindingToBridge(binding));
    } catch (e) {
      throw ShortcutException('更新快捷键配置失败: $e');
    }
  }

  Future<void> registerHotkeys(List<ShortcutBinding> shortcuts) async {
    try {
      await bridge.registerHotkeys(
        shortcuts: shortcuts.map(_mapShortcutBindingToBridge).toList(),
      );
    } catch (e) {
      throw ShortcutException('注册快捷键失败: $e');
    }
  }

Future<void> unregisterHotkeys() async {
    try {
      bridge.unregisterHotkeys();
    } catch (e) {
      throw ShortcutException('注销快捷键失败: $e');
    }
  }

  Future<String?> pollHotkeyEvent() async {
    try {
      return await bridge.pollHotkeyEvent();
    } catch (_) {
      return null;
    }
  }

  // ========== 提示词模板 ==========

  Future<List<PromptTemplate>> getPromptTemplates() async {
    try {
      final templates = await bridge.getPromptTemplates();
      return templates.map(_mapPromptTemplate).toList();
    } catch (e) {
      throw ConfigException('获取提示词模板失败: $e');
    }
  }

  Future<void> savePromptTemplate(PromptTemplate tpl) async {
    try {
      await bridge.savePromptTemplate(tpl: _mapPromptTemplateToBridge(tpl));
    } catch (e) {
      throw ConfigException('保存提示词模板失败: $e');
    }
  }

  Future<void> deletePromptTemplate(String id) async {
    try {
      await bridge.deletePromptTemplate(id: id);
    } catch (e) {
      throw ConfigException('删除提示词模板失败: $e');
    }
  }

  // ========== 剪贴板服务 ==========

  Future<String> getClipboardText() async {
    try {
      return await bridge.getClipboardText();
    } catch (e) {
      throw SystemException('获取剪贴板内容失败: $e');
    }
  }

  Future<void> setClipboardText(String text) async {
    try {
      await bridge.setClipboardText(text: text);
    } catch (e) {
      throw SystemException('设置剪贴板内容失败: $e');
    }
  }

  // ========== 托盘服务 ==========

  Future<void> initTray() async {
    try {
      await bridge.initTray();
    } catch (e) {
      throw SystemException('初始化托盘失败: $e');
    }
  }

  Future<void> showTrayNotification({
    required String title,
    required String body,
  }) async {
    try {
      await bridge.showTrayNotification(title: title, body: body);
    } catch (e) {
      throw SystemException('显示托盘通知失败: $e');
    }
  }

  // ========== 类型映射 ==========

  TranslationResult _mapTranslationResult(bridge_types.TranslationResult result) {
    return TranslationResult(
      providerId: result.providerId,
      providerName: result.providerName,
      sourceText: result.sourceText,
      translatedText: result.translatedText,
      responseTimeMs: result.responseTimeMs.toInt(),
      isSuccess: result.isSuccess,
      errorMessage: result.errorMessage,
      promptTokens: result.promptTokens.toInt(),
      completionTokens: result.completionTokens.toInt(),
      totalTokens: result.totalTokens.toInt(),
    );
  }

  ProviderConfig _mapProviderConfig(bridge_types.ProviderConfig config) {
    return ProviderConfig(
      id: config.id,
      name: config.name,
      apiKey: config.apiKey,
      apiUrl: config.apiUrl,
      model: config.model,
      authType: config.authType,
      isActive: config.isActive,
      sortOrder: config.sortOrder,
      systemPrompt: config.systemPrompt,
      createdAt: config.createdAt,
    );
  }

  bridge_types.ProviderConfig _mapProviderConfigToBridge(ProviderConfig config) {
    return bridge_types.ProviderConfig(
      id: config.id,
      name: config.name,
      apiKey: config.apiKey,
      apiUrl: config.apiUrl,
      model: config.model,
      authType: config.authType,
      isActive: config.isActive,
      sortOrder: config.sortOrder,
      systemPrompt: config.systemPrompt,
      createdAt: config.createdAt,
    );
  }

  ActiveSession _mapActiveSession(bridge_types.ActiveSession session) {
    return ActiveSession(
      lastProviderId: session.lastProviderId,
      lastCompareProviders: session.lastCompareProviders.toList(),
      lastUsed: session.lastUsed,
    );
  }

  ShortcutBinding _mapShortcutBinding(bridge_types.ShortcutBinding binding) {
    return ShortcutBinding(
      id: binding.id,
      action: binding.action,
      keyCombination: binding.keyCombination,
      enabled: binding.enabled,
    );
  }

  bridge_types.ShortcutBinding _mapShortcutBindingToBridge(ShortcutBinding binding) {
    return bridge_types.ShortcutBinding(
      id: binding.id,
      action: binding.action,
      keyCombination: binding.keyCombination,
      enabled: binding.enabled,
    );
  }

  PromptTemplate _mapPromptTemplate(bridge_types.PromptTemplate tpl) {
    return PromptTemplate(
      id: tpl.id,
      name: tpl.name,
      content: tpl.content,
      isActive: tpl.isActive,
      createdAt: tpl.createdAt,
    );
  }

  bridge_types.PromptTemplate _mapPromptTemplateToBridge(PromptTemplate tpl) {
    return bridge_types.PromptTemplate(
      id: tpl.id,
      name: tpl.name,
      content: tpl.content,
      isActive: tpl.isActive,
      createdAt: tpl.createdAt,
    );
  }
}
