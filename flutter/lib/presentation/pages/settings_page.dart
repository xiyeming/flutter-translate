import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import '../../data/datasources/ffi_datasource.dart';
import '../../data/models/provider_config.dart';
import '../../data/models/shortcut_binding.dart';
import '../../presentation/services/hotkey_service.dart';

class SettingsPage extends ConsumerStatefulWidget {
  const SettingsPage({super.key});

  @override
  ConsumerState<SettingsPage> createState() => _SettingsPageState();
}

class _SettingsPageState extends ConsumerState<SettingsPage>
    with SingleTickerProviderStateMixin {
  late final TabController _tabController;

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 4, vsync: this);
  }

  @override
  void dispose() {
    _tabController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          onPressed: () => context.go('/'),
        ),
        title: const Text('设置'),
        bottom: TabBar(
          controller: _tabController,
          isScrollable: true,
          tabAlignment: TabAlignment.start,
          tabs: const [
            Tab(icon: Icon(Icons.api), text: '厂商'),
            Tab(icon: Icon(Icons.keyboard), text: '快捷键'),
            Tab(icon: Icon(Icons.language), text: '语言'),
            Tab(icon: Icon(Icons.palette), text: '外观'),
          ],
        ),
      ),
      body: TabBarView(
        controller: _tabController,
        children: const [
          _ProviderSettingsTab(),
          _ShortcutSettingsTab(),
          _LanguageSettingsTab(),
          _ThemeSettingsTab(),
        ],
      ),
    );
  }
}

class _ProviderSettingsTab extends ConsumerStatefulWidget {
  const _ProviderSettingsTab();

  @override
  ConsumerState<_ProviderSettingsTab> createState() => _ProviderSettingsTabState();
}

class _ProviderSettingsTabState extends ConsumerState<_ProviderSettingsTab> {
  List<ProviderConfig> _providers = [];
  bool _isLoading = true;

  @override
  void initState() {
    super.initState();
    _loadProviders();
  }

  Future<void> _loadProviders() async {
    try {
      final ffi = FfiDatasource();
      final providers = await ffi.getProviders();
      if (mounted) setState(() { _providers = providers; _isLoading = false; });
    } catch (_) {
      if (mounted) setState(() => _isLoading = false);
    }
  }

  Future<void> _toggleActive(ProviderConfig p) async {
    final saved = _providers.where((s) => s.id == p.id).firstOrNull;
    final hasKey = saved?.apiKey != null && saved!.apiKey!.isNotEmpty;
    if (!(saved?.isActive ?? false) && !hasKey) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('请先配置 API Key 后再启用厂商'), duration: Duration(seconds: 2)),
        );
      }
      return;
    }
    final newActive = !(saved?.isActive ?? p.isActive);
    setState(() {
      final idx = _providers.indexWhere((x) => x.id == p.id);
      if (idx >= 0) _providers[idx] = (saved ?? p).copyWith(isActive: newActive);
      else {
        _providers.add(p.copyWith(isActive: newActive, createdAt: DateTime.now()));
      }
    });
    final ffi = FfiDatasource();
    await ffi.saveProvider((saved ?? p).copyWith(isActive: newActive));
  }

  @override
  Widget build(BuildContext context) {
    if (_isLoading) {
      return const Center(child: CircularProgressIndicator());
    }

    final defaultProviders = <String, ProviderConfig>{
      for (final p in [
        ProviderConfig(id: 'openai', name: 'OpenAI', model: 'gpt-4o-mini', authType: 'api_key', isActive: false, createdAt: DateTime.now()),
        ProviderConfig(id: 'deepl', name: 'DeepL', model: 'default', authType: 'api_key', isActive: false, createdAt: DateTime.now()),
        ProviderConfig(id: 'google', name: 'Google', model: 'nmt', authType: 'api_key', isActive: false, createdAt: DateTime.now()),
        ProviderConfig(id: 'qwen', name: 'Qwen (通义千问)', model: 'qwen-turbo', authType: 'api_key', isActive: false, createdAt: DateTime.now()),
        ProviderConfig(id: 'deepseek', name: 'DeepSeek', model: 'deepseek-chat', authType: 'api_key', isActive: false, createdAt: DateTime.now()),
        ProviderConfig(id: 'kimi', name: 'Kimi (月之暗面)', model: 'moonshot-v1-8k', authType: 'api_key', isActive: false, createdAt: DateTime.now()),
        ProviderConfig(id: 'glm', name: 'GLM (智谱)', model: 'glm-4-plus', authType: 'api_key', isActive: false, createdAt: DateTime.now()),
        ProviderConfig(id: 'anthropic', name: 'Anthropic', model: 'claude-3-haiku-20240307', authType: 'api_key', isActive: false, createdAt: DateTime.now()),
        ProviderConfig(id: 'azure', name: 'Azure OpenAI', model: 'gpt-4o-mini', authType: 'api_key', isActive: false, createdAt: DateTime.now()),
        ProviderConfig(id: 'custom', name: 'Custom (兼容 API)', model: '自定义', authType: 'api_key', isActive: false, createdAt: DateTime.now()),
      ]) p.id: p
    };

    final merged = <String, ProviderConfig>{};
    for (final p in _providers) { merged[p.id] = p; }
    for (final e in defaultProviders.entries) {
      merged.putIfAbsent(e.key, () => e.value);
    }

    final sorted = merged.values.toList()
      ..sort((a, b) => a.sortOrder.compareTo(b.sortOrder));

    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        ...sorted.map((p) => _buildProviderCard(context, p)),
        const SizedBox(height: 24),
        OutlinedButton.icon(
          onPressed: () => context.go('/settings/provider'),
          icon: const Icon(Icons.add),
          label: const Text('添加厂商'),
        ),
      ],
    );
  }

  Widget _buildProviderCard(BuildContext context, ProviderConfig p) {
    final isConfigured = _providers.any((s) => s.id == p.id && s.apiKey != null && s.apiKey!.isNotEmpty);
    final saved = _providers.where((s) => s.id == p.id).firstOrNull;
    final isActive = saved?.isActive ?? p.isActive;
    const icons = <String, IconData>{
      'openai': Icons.auto_awesome, 'deepl': Icons.translate, 'google': Icons.cloud,
      'qwen': Icons.auto_awesome, 'deepseek': Icons.auto_awesome, 'kimi': Icons.auto_awesome,
      'glm': Icons.auto_awesome, 'anthropic': Icons.auto_awesome, 'azure': Icons.cloud, 'custom': Icons.tune,
    };
    return Padding(
      padding: const EdgeInsets.only(bottom: 12),
      child: Card(
        child: Row(children: [
          Expanded(
            child: InkWell(
              onTap: () => context.go('/settings/provider?id=${p.id}&name=${Uri.encodeComponent(p.name)}&model=${Uri.encodeComponent(p.model)}'),
              child: Padding(
                padding: const EdgeInsets.fromLTRB(16, 12, 8, 12),
                child: Row(children: [
                  Icon(icons[p.id] ?? Icons.api, color: Theme.of(context).colorScheme.primary),
                  const SizedBox(width: 16),
                  Expanded(
                    child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
                      Text(p.name, style: Theme.of(context).textTheme.bodyMedium?.copyWith(fontWeight: FontWeight.w500)),
                      const SizedBox(height: 2),
                      Text('模型: ${p.model}${p.apiUrl != null && p.apiUrl!.isNotEmpty ? ' | ${p.apiUrl}' : ''}',
                        style: Theme.of(context).textTheme.bodySmall?.copyWith(color: Theme.of(context).colorScheme.onSurfaceVariant)),
                    ]),
                  ),
                  Icon(Icons.circle, size: 10, color: isConfigured ? Colors.green : Colors.grey),
                  const SizedBox(width: 8),
                ]),
              ),
            ),
          ),
          Switch(
            value: isActive,
            onChanged: (_) => _toggleActive(p),
            materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
          ),
          const SizedBox(width: 4),
          const Icon(Icons.chevron_right),
          const SizedBox(width: 12),
        ]),
      ),
    );
  }
}

class _ShortcutSettingsTab extends ConsumerStatefulWidget {
  const _ShortcutSettingsTab();

  @override
  ConsumerState<_ShortcutSettingsTab> createState() => _ShortcutSettingsTabState();
}

class _ShortcutSettingsTabState extends ConsumerState<_ShortcutSettingsTab> {
  List<ShortcutBinding> _bindings = [];
  bool _isLoading = true;

  static const _actions = {
    'translate_selected': '翻译选中文本',
    'ocr_screenshot': '截图翻译',
    'toggle_window': '显示/隐藏窗口',
  };

  @override
  void initState() {
    super.initState();
    _loadBindings();
  }

  Future<void> _loadBindings() async {
    try {
      final ffi = FfiDatasource();
      var bindings = await ffi.getShortcuts();
      if (bindings.isEmpty) {
        final defaults = [
          ShortcutBinding(id: 'translate_selected', action: 'translate_selected', keyCombination: 'Super+Alt+F', enabled: true),
          ShortcutBinding(id: 'ocr_screenshot', action: 'ocr_screenshot', keyCombination: 'Ctrl+Shift+S', enabled: true),
          ShortcutBinding(id: 'toggle_window', action: 'toggle_window', keyCombination: 'Ctrl+Shift+F', enabled: true),
        ];
        for (final b in defaults) { await ffi.updateShortcut(b); }
        bindings = defaults;
      }
      if (mounted) setState(() { _bindings = bindings; _isLoading = false; });
    } catch (_) {
      if (mounted) setState(() => _isLoading = false);
    }
  }

  Future<void> _updateAndApply() async {
    try {
      final service = HotkeyService();
      await service.updateAndReregister(_bindings);
    } catch (_) {}
  }

  Future<void> _recordShortcut(ShortcutBinding binding) async {
    final keys = await showDialog<String>(
      context: context,
      builder: (ctx) => _RecordShortcutDialog(currentKeys: binding.keyCombination),
    );
    if (keys != null && mounted) {
      setState(() {
        final idx = _bindings.indexWhere((b) => b.id == binding.id);
        _bindings[idx] = binding.copyWith(keyCombination: keys);
      });
      final ffi = FfiDatasource();
      final b = _bindings.firstWhere((b) => b.id == binding.id);
      await ffi.updateShortcut(b);
      await _updateAndApply();
    }
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    if (_isLoading) return const Center(child: CircularProgressIndicator());

    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        Text('修改后即时生效，点击快捷键组合开始录制', style: theme.textTheme.bodySmall?.copyWith(color: theme.colorScheme.onSurfaceVariant)),
        const SizedBox(height: 12),
        ..._bindings.map((b) => Card(
          child: Column(
            children: [
              SwitchListTile(
                title: Text(_actions[b.id] ?? b.action, style: theme.textTheme.bodyMedium),
                value: b.enabled,
                onChanged: (v) {
                  setState(() {
                    final idx = _bindings.indexWhere((b2) => b2.id == b.id);
                    _bindings[idx] = b.copyWith(enabled: v);
                  });
                  final ffi = FfiDatasource();
                  ffi.updateShortcut(b.copyWith(enabled: v));
                  _updateAndApply();
                },
              ),
              ListTile(
                title: const Text('快捷键', style: TextStyle(fontSize: 13)),
                trailing: GestureDetector(
                  onTap: () => _recordShortcut(b),
                  child: Container(
                    padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
                    decoration: BoxDecoration(color: theme.colorScheme.surfaceContainerHighest, borderRadius: BorderRadius.circular(6), border: Border.all(color: theme.colorScheme.outline.withValues(alpha: 0.3))),
                    child: Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Text(b.keyCombination, style: theme.textTheme.labelMedium?.copyWith(fontFamily: 'monospace')),
                        const SizedBox(width: 4),
                        Icon(Icons.edit, size: 14, color: theme.colorScheme.onSurfaceVariant),
                      ],
                    ),
                  ),
                ),
              ),
            ],
          ),
        )),
      ],
    );
  }
}

class _RecordShortcutDialog extends StatefulWidget {
  final String currentKeys;
  const _RecordShortcutDialog({required this.currentKeys});

  @override
  State<_RecordShortcutDialog> createState() => _RecordShortcutDialogState();
}

class _RecordShortcutDialogState extends State<_RecordShortcutDialog> {
  final _modifiers = <String>[];
  String _key = '';
  bool _recording = false;
  bool _done = false;

  String get _display {
    final parts = [..._modifiers.toSet()];
    if (_key.isNotEmpty) parts.add(_key);
    return parts.join(' + ');
  }

  @override
  void initState() {
    super.initState();
    HardwareKeyboard.instance.addHandler(_handleKey);
  }

  @override
  void dispose() {
    HardwareKeyboard.instance.removeHandler(_handleKey);
    super.dispose();
  }

  bool _handleKey(KeyEvent event) {
    if (_done) return false;
    if (event is! KeyDownEvent) return false;

    final label = event.logicalKey.keyLabel;

    final isMod = event.logicalKey == LogicalKeyboardKey.controlLeft ||
        event.logicalKey == LogicalKeyboardKey.controlRight ||
        event.logicalKey == LogicalKeyboardKey.shiftLeft ||
        event.logicalKey == LogicalKeyboardKey.shiftRight ||
        event.logicalKey == LogicalKeyboardKey.altLeft ||
        event.logicalKey == LogicalKeyboardKey.altRight ||
        event.logicalKey == LogicalKeyboardKey.metaLeft;

    if (isMod) {
      setState(() {
        if (event.logicalKey == LogicalKeyboardKey.controlLeft || event.logicalKey == LogicalKeyboardKey.controlRight) _modifiers.add('Ctrl');
        else if (event.logicalKey == LogicalKeyboardKey.shiftLeft || event.logicalKey == LogicalKeyboardKey.shiftRight) _modifiers.add('Shift');
        else if (event.logicalKey == LogicalKeyboardKey.altLeft || event.logicalKey == LogicalKeyboardKey.altRight) _modifiers.add('Alt');
        else if (event.logicalKey == LogicalKeyboardKey.metaLeft) _modifiers.add('Super');
        _recording = true;
      });
      return true;
    }

    if (label.isNotEmpty && label.length <= 4 && _recording) {
      final hw = HardwareKeyboard.instance;
      if (hw.isControlPressed) _modifiers.add('Ctrl');
      if (hw.isShiftPressed) _modifiers.add('Shift');
      if (hw.isAltPressed) _modifiers.add('Alt');
      if (hw.isMetaPressed) _modifiers.add('Super');
      _key = label.length == 1 ? label.toUpperCase() : label;
      _done = true;
      Future.delayed(Duration.zero, () => Navigator.pop(context, _display));
      return true;
    }

    return false;
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return AlertDialog(
      title: const Text('录制快捷键'),
      content: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text('同时按下组合键 (Ctrl/Shift/Alt/Super + 字母)', style: theme.textTheme.bodySmall),
          const SizedBox(height: 16),
          Container(
            width: double.infinity,
            padding: const EdgeInsets.symmetric(vertical: 20),
            decoration: BoxDecoration(
              border: Border.all(color: _recording ? theme.colorScheme.primary : theme.colorScheme.outline, width: 2),
              borderRadius: BorderRadius.circular(12),
              color: _recording ? theme.colorScheme.primary.withValues(alpha: 0.05) : theme.colorScheme.surfaceContainerHighest,
            ),
            alignment: Alignment.center,
            child: Text(
              _recording ? _display : (widget.currentKeys.isNotEmpty ? widget.currentKeys : '按下组合键...'),
              style: theme.textTheme.titleMedium?.copyWith(
                fontFamily: _recording ? 'monospace' : null,
                fontWeight: FontWeight.bold,
              ),
            ),
          ),
        ],
      ),
      actions: [
        TextButton(onPressed: () { _done = true; Navigator.pop(context, null); }, child: const Text('取消')),
        TextButton(onPressed: () { _done = true; Navigator.pop(context, widget.currentKeys); }, child: const Text('保持当前')),
      ],
    );
  }
}

class _LanguageSettingsTab extends ConsumerWidget {
  const _LanguageSettingsTab();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        SwitchListTile(
          title: const Text('自动检测语言'),
          subtitle: const Text('翻译时自动识别源语言'),
          value: true,
          onChanged: (_) {},
        ),
        const Divider(),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
          child: Text(
            '常用语言',
            style: Theme.of(context).textTheme.titleSmall,
          ),
        ),
        ...['中文', '英语', '日语', '韩语', '法语'].map((lang) {
          return CheckboxListTile(
            title: Text(lang),
            value: true,
            onChanged: (_) {},
          );
        }),
      ],
    );
  }
}

class _ThemeSettingsTab extends ConsumerStatefulWidget {
  const _ThemeSettingsTab();

  @override
  ConsumerState<_ThemeSettingsTab> createState() => _ThemeSettingsTabState();
}

class _ThemeSettingsTabState extends ConsumerState<_ThemeSettingsTab> {
  String _themeMode = 'system';

  @override
  Widget build(BuildContext context) {
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        RadioGroup<String>(
          groupValue: _themeMode,
          onChanged: (v) { if (v != null) setState(() => _themeMode = v); },
          child: Column(
            children: [
              RadioListTile<String>(
                title: const Text('跟随系统'),
                value: 'system',
              ),
              RadioListTile<String>(
                title: const Text('浅色模式'),
                value: 'light',
              ),
              RadioListTile<String>(
                title: const Text('深色模式'),
                value: 'dark',
              ),
            ],
          ),
        ),
      ],
    );
  }
}