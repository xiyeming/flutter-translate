import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:window_manager/window_manager.dart';
import '../../data/models/translation_result.dart';
import '../../data/models/prompt_template.dart';
import '../../data/models/provider_config.dart';
import '../../data/datasources/ffi_datasource.dart';
import '../services/hotkey_service.dart';

class FloatingPage extends ConsumerStatefulWidget {
  const FloatingPage({super.key});

  @override
  ConsumerState<FloatingPage> createState() => _FloatingPageState();
}

class _FloatingPageState extends ConsumerState<FloatingPage> {
  final _textController = TextEditingController();
  String _sourceLang = 'auto';
  String _targetLang = 'zh';
  final _results = <String, TranslationResult>{};
  bool _isTranslating = false;
  bool _providerPanelOpen = false;
  String? _activePromptId;
  String? _activePromptContent;

  static const _languages = <String, String>{
    'auto': '自动检测', 'zh': '中文', 'en': '英语', 'ja': '日语', 'ko': '韩语',
    'fr': '法语', 'de': '德语', 'es': '西班牙语', 'ru': '俄语', 'pt': '葡萄牙语',
  };

  Map<String, String> _providers = const {
    'openai': 'OpenAI', 'deepl': 'DeepL', 'google': 'Google',
    'qwen': 'Qwen', 'deepseek': 'DeepSeek', 'kimi': 'Kimi', 'glm': 'GLM',
    'anthropic': 'Anthropic', 'azure': 'Azure', 'custom': 'Custom',
  };
  final _selectedProviders = <String>{};
  List<ProviderConfig> _savedProviders = [];
  List<PromptTemplate> _promptTemplates = [];
  bool _isLoading = true;
  bool _isOcrProcessing = false;
  final _lastHotkeyTime = <String, DateTime>{};

  final _ffi = FfiDatasource();

  @override
  void initState() {
    super.initState();
    _loadAll();
    _listenHotkeys();
  }

  void _listenHotkeys() {
    HotkeyService().hotkeyStream.listen((action) async {
      final now = DateTime.now();
      final last = _lastHotkeyTime[action];
      if (last != null && now.difference(last).inMilliseconds < 500) return;
      _lastHotkeyTime[action] = now;
      if (action == 'translate_selected') {
        await _handleTranslateSelected();
      } else if (action == 'toggle_window') {
        final visible = await windowManager.isVisible();
        if (visible) { await windowManager.hide(); } else { await windowManager.show(); await windowManager.focus(); }
      } else if (action == 'ocr_screenshot') {
        await _handleOcrScreenshot();
      }
    });
  }

  Future<void> _handleTranslateSelected() async {
    try {
      final clip = await _ffi.getClipboardText();
      if (clip.isNotEmpty && mounted) {
        _textController.text = clip;
        setState(() { _results.clear(); });
        _translate();
        await windowManager.show();
        await windowManager.focus();
      }
    } catch (_) {}
  }

  Future<void> _handleOcrScreenshot() async {
    if (_isOcrProcessing) return;
    setState(() => _isOcrProcessing = true);
    try {
      final text = await _ffi.ocrScreenshot();
      if (text.isNotEmpty && mounted) {
        _textController.text = text;
        setState(() { _results.clear(); _isOcrProcessing = false; });
        _translate();
        await windowManager.show();
        await windowManager.focus();
      } else {
        setState(() => _isOcrProcessing = false);
      }
    } catch (e) {
      setState(() => _isOcrProcessing = false);
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('截图识别失败: $e'), duration: const Duration(seconds: 2)),
        );
      }
    }
  }

  Future<void> _loadAll() async {
    setState(() => _isLoading = true);
    await _reloadProviders();
    await _reloadSession();
    await _reloadPrompts();
    if (mounted) setState(() => _isLoading = false);
  }

  Future<void> _reloadProviders() async {
    try {
      final providers = await _ffi.getProviders();
      if (mounted) setState(() { _savedProviders = providers; for (final p in providers) { _providers[p.id] = p.name; } });
    } catch (_) {}
  }

  Future<void> _reloadSession() async {
    try {
      final session = await _ffi.getActiveSession();
      if (mounted && session.lastCompareProviders.isNotEmpty) {
        final activeIds = _savedProviders.where((p) => p.isActive).map((p) => p.id).toSet();
        setState(() {
          _selectedProviders.clear();
          _selectedProviders.addAll(session.lastCompareProviders.where((id) => activeIds.contains(id)));
          if (_selectedProviders.isEmpty && activeIds.isNotEmpty) _selectedProviders.add(activeIds.first);
        });
      }
    } catch (_) {}
  }

  Future<void> _reloadPrompts() async {
    try {
      final templates = await _ffi.getPromptTemplates();
      if (mounted) {
        setState(() {
          _promptTemplates = templates;
          final active = templates.where((t) => t.isActive).firstOrNull;
          if (active != null) { _activePromptId = active.id; _activePromptContent = active.content; }
        });
      }
    } catch (_) {}
  }

  @override
  void dispose() {
    _textController.dispose();
    super.dispose();
  }

  Future<void> _saveSession() async {
    try { await _ffi.updateSession(providerId: _selectedProviders.first, compareProviders: _selectedProviders.toList()); } catch (_) {}
  }

  Future<void> _translate() async {
    if (_textController.text.trim().isEmpty || _selectedProviders.isEmpty) return;
    setState(() => _isTranslating = true);
    try {
      if (_selectedProviders.length == 1) {
        final pid = _selectedProviders.first;
        final r = await _ffi.translate(text: _textController.text.trim(), sourceLang: _sourceLang, targetLang: _targetLang, providerId: pid, systemPromptOverride: _activePromptContent);
        setState(() { _results[pid] = r; _isTranslating = false; });
      } else {
        final results = await _ffi.translateCompare(text: _textController.text.trim(), sourceLang: _sourceLang, targetLang: _targetLang, providerIds: _selectedProviders.toList(), systemPromptOverride: _activePromptContent);
        setState(() { for (final r in results) { _results[r.providerId] = r; } _isTranslating = false; });
      }
    } catch (e) {
      setState(() {
        for (final id in _selectedProviders) {
          _results[id] = TranslationResult(providerId: id, providerName: _providers[id] ?? id, sourceText: _textController.text.trim(), translatedText: '翻译失败: $e', responseTimeMs: 0, isSuccess: false, errorMessage: e.toString());
        }
        _isTranslating = false;
      });
    }
  }

  void _swapLanguages() {
    if (_sourceLang == 'auto') return;
    setState(() { final t = _sourceLang; _sourceLang = _targetLang; _targetLang = t; });
  }

  void _toggleProvider(String id) {
    setState(() {
      if (_selectedProviders.contains(id)) { if (_selectedProviders.length > 1) { _selectedProviders.remove(id); _results.remove(id); } }
      else { _selectedProviders.add(id); }
    });
    _saveSession();
  }

  // ========== UI ==========

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    if (_isLoading) return Scaffold(body: const Center(child: CircularProgressIndicator()));

    return Scaffold(
      backgroundColor: theme.colorScheme.surface,
      body: SafeArea(
        top: false,
        child: Column(
          children: [
            _buildHeader(theme),
            const Divider(height: 1),
            Expanded(
              child: SingleChildScrollView(
                padding: const EdgeInsets.fromLTRB(12, 8, 12, 16),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    _buildLanguageBar(theme),
                    const SizedBox(height: 10),
                    _buildPromptSelector(theme),
                    if (_isOcrProcessing) _buildOcrStatus(theme),
                    const SizedBox(height: 10),
                    _buildProviderChips(theme),
                    if (_providerPanelOpen) _buildProviderPanel(theme),
                    const SizedBox(height: 10),
                    _buildInputArea(theme),
                    const SizedBox(height: 10),
                    _buildTranslateButton(theme),
                    if (_results.isNotEmpty) ...[
                      const SizedBox(height: 10),
                      ..._selectedProviders.map((id) => _buildResultCard(theme, id)),
                    ],
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildHeader(ThemeData theme) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
      child: Row(children: [
        Icon(Icons.translate, size: 20, color: theme.colorScheme.primary),
        const SizedBox(width: 8),
        Text('AI 翻译', style: theme.textTheme.titleSmall?.copyWith(fontWeight: FontWeight.w600)),
        const Spacer(),
        IconButton(icon: const Icon(Icons.settings, size: 18), tooltip: '设置', onPressed: () => context.go('/settings')),
      ]),
    );
  }

  Widget _buildCompactDropdown<T>({
    required String label,
    required T? value,
    required List<PopupMenuEntry<T>> items,
    required ValueChanged<T?> onSelected,
    Widget? icon,
  }) {
    return PopupMenuButton<T>(
      offset: const Offset(0, 36),
      position: PopupMenuPosition.under,
      onSelected: onSelected,
      itemBuilder: (_) => items,
      child: Container(
        height: 36,
        padding: const EdgeInsets.symmetric(horizontal: 10),
        decoration: BoxDecoration(border: Border.all(color: Colors.grey.withValues(alpha: 0.4)), borderRadius: BorderRadius.circular(6)),
        child: Row(children: [
          if (icon != null) ...[icon, const SizedBox(width: 4)],
          Expanded(child: Text(_labelForValue<T>(label, value), style: const TextStyle(fontSize: 13), overflow: TextOverflow.ellipsis)),
          const Icon(Icons.arrow_drop_down, size: 18, color: Colors.grey),
        ]),
      ),
    );
  }

  String _labelForValue<T>(String label, T? value) {
    if (value == null) return label;
    if (value is String && _languages.containsKey(value)) return _languages[value]!;
    if (value is String && _activePromptId != null) {
      return _promptTemplates.where((t) => t.id == value).firstOrNull?.name ?? label;
    }
    return label;
  }

  Widget _buildLanguageBar(ThemeData theme) {
    return Row(children: [
      Expanded(child: _buildCompactDropdown<String>(label: '源语言', value: _sourceLang, icon: Icon(Icons.language, size: 14, color: theme.colorScheme.onSurfaceVariant),
        items: _languages.entries.where((e) => e.key != _targetLang).map((e) => PopupMenuItem(value: e.key, child: Text(e.value, style: const TextStyle(fontSize: 13)))).toList(),
        onSelected: (v) { if (v != null) setState(() => _sourceLang = v); })),
      IconButton(icon: const Icon(Icons.swap_horiz, size: 18), tooltip: '切换', visualDensity: VisualDensity.compact, onPressed: _swapLanguages),
      Expanded(child: _buildCompactDropdown<String>(label: '目标语言', value: _targetLang, icon: Icon(Icons.translate, size: 14, color: theme.colorScheme.onSurfaceVariant),
        items: _languages.entries.where((e) => e.key != 'auto' && e.key != _sourceLang).map((e) => PopupMenuItem(value: e.key, child: Text(e.value, style: const TextStyle(fontSize: 13)))).toList(),
        onSelected: (v) { if (v != null) setState(() => _targetLang = v); })),
    ]);
  }

  Widget _buildPromptSelector(ThemeData theme) {
    final hasActive = _activePromptId != null && _activePromptId != '__default__';
    final selectedName = hasActive ? _promptTemplates.where((t) => t.id == _activePromptId).firstOrNull?.name : null;
    final defaultValue = '__default__';
    final displayValue = _activePromptId == null || _activePromptId == defaultValue ? null : _activePromptId;
    final items = <PopupMenuEntry<String>>[
      PopupMenuItem<String>(value: defaultValue, child: Row(children: [
        SizedBox(width: 18, child: !hasActive ? Icon(Icons.check, size: 16, color: theme.colorScheme.primary) : null),
        const SizedBox(width: 4),
        const Text('使用厂商默认提示词', style: TextStyle(fontSize: 13)),
      ])),
      ..._promptTemplates.map((t) => PopupMenuItem<String>(value: t.id, child: Row(children: [
        SizedBox(width: 18, child: displayValue == t.id ? Icon(Icons.check, size: 16, color: theme.colorScheme.primary) : null),
        const SizedBox(width: 4),
        Text(t.name, style: const TextStyle(fontSize: 13)),
      ]))),
    ];
    return Container(
      height: 36,
      decoration: BoxDecoration(border: Border.all(color: hasActive ? theme.colorScheme.primary.withValues(alpha: 0.4) : Colors.grey.withValues(alpha: 0.4)), borderRadius: BorderRadius.circular(6)),
      child: Row(children: [
        Padding(padding: const EdgeInsets.symmetric(horizontal: 8), child: Icon(Icons.auto_awesome, size: 14, color: hasActive ? theme.colorScheme.primary : theme.colorScheme.onSurfaceVariant)),
        Expanded(
          child: PopupMenuButton<String>(
            offset: const Offset(0, 36), position: PopupMenuPosition.under,
            onSelected: (v) {
              setState(() {
                if (v == defaultValue) { _activePromptId = v; _activePromptContent = null; }
                else { _activePromptId = v; _activePromptContent = _promptTemplates.where((t) => t.id == v).firstOrNull?.content; }
              });
            },
            itemBuilder: (_) => items,
            child: Align(alignment: Alignment.centerLeft, child: Text(selectedName ?? '使用厂商默认提示词', style: TextStyle(fontSize: 13, color: selectedName != null ? theme.colorScheme.onSurface : Colors.grey))),
          ),
        ),
        IconButton(icon: const Icon(Icons.add_circle_outline, size: 18), tooltip: '管理', visualDensity: VisualDensity.compact, onPressed: () => _showPromptManager()),
      ]),
    );
  }

  Widget _buildProviderChips(ThemeData theme) {
    return Wrap(spacing: 6, runSpacing: 4, children: [
      ..._selectedProviders.map((id) => Chip(
        label: Text(_providers[id] ?? id, style: theme.textTheme.labelSmall),
        deleteIcon: const Icon(Icons.close, size: 14),
        onDeleted: _selectedProviders.length > 1 ? () => setState(() { _selectedProviders.remove(id); _results.remove(id); }) : null,
        materialTapTargetSize: MaterialTapTargetSize.shrinkWrap, visualDensity: VisualDensity.compact,
      )),
      ActionChip(
        avatar: Icon(_providerPanelOpen ? Icons.expand_less : Icons.expand_more, size: 16),
        label: Text(_providerPanelOpen ? '收起' : '选择厂商', style: theme.textTheme.labelSmall),
        onPressed: () => setState(() => _providerPanelOpen = !_providerPanelOpen),
        materialTapTargetSize: MaterialTapTargetSize.shrinkWrap, visualDensity: VisualDensity.compact,
      ),
    ]);
  }

  Widget _buildOcrStatus(ThemeData theme) {
    return Padding(
      padding: const EdgeInsets.only(top: 4),
      child: Row(children: [
        const SizedBox(width: 16, height: 16, child: CircularProgressIndicator(strokeWidth: 2)),
        const SizedBox(width: 8),
        Text('截图识别中...', style: theme.textTheme.bodySmall?.copyWith(color: theme.colorScheme.primary)),
      ]),
    );
  }

  Widget _buildProviderPanel(ThemeData theme) {
    final activeIds = _savedProviders.isEmpty
        ? _providers.keys.toSet()  // 未保存时所有厂商默认可用
        : _savedProviders.where((p) => p.isActive).map((p) => p.id).toSet();
    final activeEntries = _providers.entries.where((e) => activeIds.contains(e.key)).toList();
    if (activeEntries.isEmpty) {
      return Card(
        margin: const EdgeInsets.only(top: 4),
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Text('暂无启用的厂商，请在设置中启用', style: theme.textTheme.bodySmall?.copyWith(color: theme.colorScheme.onSurfaceVariant)),
        ),
      );
    }
    return Card(
      margin: const EdgeInsets.only(top: 4),
      child: Padding(padding: const EdgeInsets.all(6), child: Wrap(
        spacing: 0, runSpacing: 0,
        children: activeEntries.map((e) => SizedBox(width: 150, child: CheckboxListTile(
          value: _selectedProviders.contains(e.key),
          title: Text(e.value, style: theme.textTheme.bodySmall),
          controlAffinity: ListTileControlAffinity.leading,
          contentPadding: const EdgeInsets.symmetric(horizontal: 2),
          dense: true,
          visualDensity: VisualDensity.compact,
          onChanged: (_) => _toggleProvider(e.key),
        ))).toList(),
      )),
    );
  }

  Widget _buildInputArea(ThemeData theme) {
    return Focus(
      onKeyEvent: (node, event) {
        if (event is KeyDownEvent &&
            event.logicalKey == LogicalKeyboardKey.enter &&
            !HardwareKeyboard.instance.isShiftPressed &&
            !HardwareKeyboard.instance.isControlPressed &&
            !HardwareKeyboard.instance.isAltPressed) {
          _translate();
          return KeyEventResult.handled;
        }
        return KeyEventResult.ignored;
      },
      child: TextField(
        controller: _textController,
        maxLines: 4, minLines: 2,
        decoration: InputDecoration(
          hintText: '输入要翻译的文本... (Shift+Enter 换行)',
          border: OutlineInputBorder(borderRadius: BorderRadius.circular(10)),
          contentPadding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
          suffixIcon: _textController.text.isNotEmpty ? IconButton(icon: const Icon(Icons.clear, size: 18), onPressed: () { _textController.clear(); setState(() => _results.clear()); }) : null,
        ),
        onChanged: (_) => setState(() {}),
      ),
    );
  }

  Widget _buildTranslateButton(ThemeData theme) {
    return FilledButton.icon(
      onPressed: _isTranslating || _selectedProviders.isEmpty ? null : _translate,
      icon: _isTranslating ? const SizedBox(width: 16, height: 16, child: CircularProgressIndicator(strokeWidth: 2)) : const Icon(Icons.translate, size: 18),
      label: Text(_isTranslating ? '翻译中...' : _selectedProviders.length == 1 ? '翻译' : '翻译 (${_selectedProviders.length}个厂商)'),
    );
  }

  Widget _buildResultCard(ThemeData theme, String providerId) {
    final result = _results[providerId];
    final providerName = _providers[providerId] ?? providerId;

    if (result == null) {
      if (_isTranslating) return Card(margin: const EdgeInsets.only(bottom: 6), child: Padding(padding: const EdgeInsets.all(10), child: Row(children: [
        SizedBox(width: 14, height: 14, child: CircularProgressIndicator(strokeWidth: 2, color: theme.colorScheme.primary)),
        const SizedBox(width: 8), Text('$providerName 翻译中...', style: theme.textTheme.bodySmall),
      ])));
      return const SizedBox.shrink();
    }

    final isError = !result.isSuccess;
    final borderColor = isError ? theme.colorScheme.error.withValues(alpha: 0.3) : theme.colorScheme.primary.withValues(alpha: 0.2);

    return Container(
      margin: const EdgeInsets.only(bottom: 6),
      decoration: BoxDecoration(border: Border.all(color: borderColor), borderRadius: BorderRadius.circular(10)),
      padding: const EdgeInsets.all(10),
      child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        Row(children: [
          Icon(isError ? Icons.error_outline : Icons.check_circle, size: 14, color: isError ? theme.colorScheme.error : theme.colorScheme.primary),
          const SizedBox(width: 6),
          Text(providerName, style: theme.textTheme.labelMedium?.copyWith(fontWeight: FontWeight.w600, color: isError ? theme.colorScheme.error : theme.colorScheme.primary)),
          const Spacer(),
          if (result.responseTimeMs > 0)
            Container(padding: const EdgeInsets.symmetric(horizontal: 5, vertical: 1), decoration: BoxDecoration(color: result.responseTimeMs < 500 ? Colors.green.withValues(alpha: 0.1) : result.responseTimeMs < 1000 ? Colors.orange.withValues(alpha: 0.1) : Colors.red.withValues(alpha: 0.1), borderRadius: BorderRadius.circular(4)),
              child: Text('${result.responseTimeMs}ms', style: const TextStyle(fontSize: 10, color: Colors.grey))),
          if (result.totalTokens > 0) Padding(padding: const EdgeInsets.only(left: 4), child: Text('${result.totalTokens}t', style: const TextStyle(fontSize: 10, color: Colors.grey))),
          const SizedBox(width: 2),
          IconButton(icon: const Icon(Icons.copy, size: 14), tooltip: '复制', constraints: const BoxConstraints(minWidth: 24, minHeight: 24), padding: EdgeInsets.zero, onPressed: () { _ffi.setClipboardText(result.translatedText); }),
        ]),
        const SizedBox(height: 4),
        SelectableText(result.translatedText, style: theme.textTheme.bodySmall),
      ]),
    );
  }

  // ========== 提示词模板管理 ==========

  Future<void> _showPromptManager() async {
    final result = await showModalBottomSheet<bool>(
      context: context, isScrollControlled: true,
      builder: (_) => _PromptManagerSheet(templates: _promptTemplates, onSave: (tpl) => _savePrompt(tpl), onDelete: (id) => _deletePrompt(id), onActivate: (id) => _activatePrompt(id)),
    );
    if (result == true) await _loadPrompts();
  }

  Future<void> _savePrompt(PromptTemplate tpl) async { await _ffi.savePromptTemplate(tpl); }
  Future<void> _deletePrompt(String id) async { await _ffi.deletePromptTemplate(id); }
  Future<void> _activatePrompt(String id) async {
    final tpl = _promptTemplates.firstWhere((t) => t.id == id);
    await _ffi.savePromptTemplate(tpl.copyWith(isActive: true));
  }

  Future<void> _loadPrompts() async {
    try {
      final templates = await _ffi.getPromptTemplates();
      if (mounted) setState(() { _promptTemplates = templates; final a = templates.where((t) => t.isActive).firstOrNull; _activePromptId = a?.id; _activePromptContent = a?.content; });
    } catch (_) {}
  }
}

// ========== 提示词模板管理 BottomSheet ==========

class _PromptManagerSheet extends StatefulWidget {
  final List<PromptTemplate> templates;
  final Future<void> Function(PromptTemplate) onSave;
  final Future<void> Function(String) onDelete;
  final Future<void> Function(String) onActivate;

  _PromptManagerSheet({required this.templates, required this.onSave, required this.onDelete, required this.onActivate});

  @override
  State<_PromptManagerSheet> createState() => _PromptManagerSheetState();
}

class _PromptManagerSheetState extends State<_PromptManagerSheet> {
  Future<void> _addOrEdit({PromptTemplate? existing}) async {
    final nameCtrl = TextEditingController(text: existing?.name ?? '');
    final contentCtrl = TextEditingController(text: existing?.content ?? '');
    final result = await showDialog<bool>(context: context, builder: (ctx) => AlertDialog(
      title: Text(existing != null ? '编辑提示词' : '新增提示词'),
      content: SingleChildScrollView(child: Column(mainAxisSize: MainAxisSize.min, children: [
        TextField(controller: nameCtrl, decoration: const InputDecoration(labelText: '名称', hintText: '例如: 翻译助手', border: OutlineInputBorder())),
        const SizedBox(height: 12),
        TextField(controller: contentCtrl, maxLines: 6, minLines: 3, decoration: const InputDecoration(labelText: '提示词内容', hintText: '输入系统提示词...', border: OutlineInputBorder(), alignLabelWithHint: true)),
      ])),
      actions: [TextButton(onPressed: () => Navigator.pop(ctx, false), child: const Text('取消')), FilledButton(onPressed: () => Navigator.pop(ctx, true), child: const Text('保存'))],
    ));
    if (result == true && nameCtrl.text.trim().isNotEmpty && contentCtrl.text.trim().isNotEmpty) {
      await widget.onSave(PromptTemplate(id: existing?.id ?? DateTime.now().millisecondsSinceEpoch.toString(), name: nameCtrl.text.trim(), content: contentCtrl.text.trim(), isActive: existing?.isActive ?? false, createdAt: existing?.createdAt ?? DateTime.now()));
      if (mounted) Navigator.pop(context, true);
    }
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return DraggableScrollableSheet(initialChildSize: 0.6, minChildSize: 0.3, maxChildSize: 0.9, expand: false, builder: (_, scrollCtrl) => Padding(
      padding: const EdgeInsets.all(16),
      child: Column(children: [
        Center(child: Container(width: 32, height: 4, decoration: BoxDecoration(color: theme.colorScheme.onSurfaceVariant.withValues(alpha: 0.3), borderRadius: BorderRadius.circular(2)))),
        const SizedBox(height: 16),
        Row(children: [Text('提示词模板管理', style: theme.textTheme.titleSmall), const Spacer(), IconButton(icon: const Icon(Icons.add, size: 20), tooltip: '新增', onPressed: () => _addOrEdit())]),
        const SizedBox(height: 8),
        if (widget.templates.isEmpty)
          Expanded(child: Center(child: Text('暂无提示词模板，点击 + 新增', style: theme.textTheme.bodySmall?.copyWith(color: theme.colorScheme.onSurfaceVariant))))
        else
          Expanded(child: ListView.builder(controller: scrollCtrl, itemCount: widget.templates.length, itemBuilder: (_, i) {
            final t = widget.templates[i];
            return Card(child: ListTile(
              leading: t.isActive ? Icon(Icons.check_circle, color: theme.colorScheme.primary, size: 20) : const Icon(Icons.circle_outlined, size: 20),
              title: Text(t.name, style: theme.textTheme.bodyMedium),
              subtitle: Text(t.content, maxLines: 1, overflow: TextOverflow.ellipsis, style: theme.textTheme.bodySmall?.copyWith(color: theme.colorScheme.onSurfaceVariant)),
              trailing: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  if (!t.isActive)
                    InkWell(
                      onTap: () async { await widget.onActivate(t.id); if (mounted) Navigator.pop(context, true); },
                      child: Padding(padding: const EdgeInsets.all(4), child: Icon(Icons.check_circle_outline, size: 20, color: theme.colorScheme.primary)),
                    ),
                  InkWell(
                    onTap: () => _addOrEdit(existing: t),
                    child: const Padding(padding: EdgeInsets.all(4), child: Icon(Icons.edit, size: 20)),
                  ),
                  InkWell(
                    onTap: () async {
                      final confirm = await showDialog<bool>(context: context, builder: (c) => AlertDialog(title: const Text('确认删除'), content: Text('删除 "${t.name}"？'), actions: [TextButton(onPressed: () => Navigator.pop(c, false), child: const Text('取消')), FilledButton(onPressed: () => Navigator.pop(c, true), child: const Text('删除'))]));
                      if (confirm == true) { await widget.onDelete(t.id); if (mounted) Navigator.pop(context, true); }
                    },
                    child: Padding(padding: const EdgeInsets.all(4), child: Icon(Icons.delete_outline, size: 20, color: theme.colorScheme.error)),
                  ),
                ],
              ),
            ));
          })),
      ]),
    ));
  }
}
