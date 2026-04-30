import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import '../../data/models/translation_result.dart';
import '../../data/datasources/ffi_datasource.dart';

class ComparePage extends ConsumerStatefulWidget {
  const ComparePage({super.key});

  @override
  ConsumerState<ComparePage> createState() => _ComparePageState();
}

class _ComparePageState extends ConsumerState<ComparePage> {
  final _textController = TextEditingController();
  final _compareResults = <String, TranslationResult>{};
  final _selectedProviders = <String>{'openai', 'deepl'};
  bool _isComparing = false;

  final _allProviders = <String, String>{
    'openai': 'OpenAI',
    'deepl': 'DeepL',
    'google': 'Google',
    'qwen': 'Qwen',
    'deepseek': 'DeepSeek',
    'kimi': 'Kimi',
    'glm': 'GLM',
    'anthropic': 'Anthropic',
    'azure': 'Azure',
    'custom': 'Custom',
  };

  final _ffi = FfiDatasource();

  @override
  void dispose() {
    _textController.dispose();
    super.dispose();
  }

  Future<void> _startCompare() async {
    if (_textController.text.trim().isEmpty || _selectedProviders.isEmpty) return;

    setState(() => _isComparing = true);

    try {
      final results = await _ffi.translateCompare(
        text: _textController.text.trim(),
        sourceLang: 'auto',
        targetLang: 'zh',
        providerIds: _selectedProviders.toList(),
      );
      setState(() {
        _compareResults.clear();
        for (final r in results) {
          _compareResults[r.providerId] = r;
        }
        _isComparing = false;
      });
    } catch (e) {
      setState(() {
        for (final id in _selectedProviders) {
          _compareResults[id] = TranslationResult(
            providerId: id,
            providerName: _allProviders[id] ?? id,
            sourceText: _textController.text.trim(),
            translatedText: '翻译失败: $e',
            responseTimeMs: 0,
            isSuccess: false,
            errorMessage: e.toString(),
          );
        }
        _isComparing = false;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          onPressed: () => context.go('/'),
        ),
        title: const Text('多厂商对比'),
        actions: [
          PopupMenuButton<String>(
            icon: const Icon(Icons.filter_list),
            onSelected: (id) {
              setState(() {
                if (_selectedProviders.contains(id)) {
                  if (_selectedProviders.length > 1) {
                    _selectedProviders.remove(id);
                    _compareResults.remove(id);
                  }
                } else {
                  _selectedProviders.add(id);
                }
              });
            },
            itemBuilder: (context) {
              return _allProviders.entries.map((e) {
                return PopupMenuItem<String>(
                  value: e.key,
                  child: Row(
                    children: [
                      Checkbox(
                        value: _selectedProviders.contains(e.key),
                        onChanged: null,
                      ),
                      Text(e.value),
                    ],
                  ),
                );
              }).toList();
            },
          ),
        ],
      ),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          children: [
            TextField(
              controller: _textController,
              maxLines: 3,
              decoration: InputDecoration(
                hintText: '输入对比翻译文本...',
                border: OutlineInputBorder(
                  borderRadius: BorderRadius.circular(12),
                ),
              ),
            ),
            const SizedBox(height: 12),

            Wrap(
              spacing: 8,
              children: _selectedProviders.map((id) {
                return Chip(
                  avatar: Icon(Icons.api, size: 16, color: theme.colorScheme.primary),
                  label: Text(_allProviders[id] ?? id),
                  deleteIcon: const Icon(Icons.close, size: 16),
                  onDeleted: _selectedProviders.length > 1
                      ? () {
                          setState(() {
                            _selectedProviders.remove(id);
                            _compareResults.remove(id);
                          });
                        }
                      : null,
                );
              }).toList(),
            ),
            const SizedBox(height: 12),

            SizedBox(
              width: double.infinity,
              child: FilledButton.icon(
                onPressed: _isComparing || _selectedProviders.isEmpty
                    ? null
                    : _startCompare,
                icon: _isComparing
                    ? const SizedBox(
                        width: 16,
                        height: 16,
                        child: CircularProgressIndicator(strokeWidth: 2),
                      )
                    : const Icon(Icons.compare),
                label: Text(_isComparing
                    ? '对比中...'
                    : '开始对比 (${_selectedProviders.length}个厂商)'),
              ),
            ),
            const SizedBox(height: 16),

            if (_compareResults.isNotEmpty)
              Expanded(
                child: ListView(
                  children: _selectedProviders.map((id) {
                    final result = _compareResults[id];
                    if (result == null) {
                      if (_isComparing) {
                        return Card(
                          margin: const EdgeInsets.only(bottom: 12),
                          child: Padding(
                            padding: const EdgeInsets.all(16),
                            child: Row(
                              children: [
                                SizedBox(
                                  width: 16,
                                  height: 16,
                                  child: CircularProgressIndicator(
                                    strokeWidth: 2,
                                    color: theme.colorScheme.primary,
                                  ),
                                ),
                                const SizedBox(width: 8),
                                Text(
                                  '${_allProviders[id] ?? id} 翻译中...',
                                  style: theme.textTheme.bodySmall,
                                ),
                              ],
                            ),
                          ),
                        );
                      }
                      return const SizedBox.shrink();
                    }
                    return _buildResultCard(theme, id, result);
                  }).toList(),
                ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildResultCard(ThemeData theme, String providerId, TranslationResult result) {
    final providerName = _allProviders[providerId] ?? providerId;
    final isError = !result.isSuccess;
    final borderColor = isError
        ? theme.colorScheme.error.withValues(alpha: 0.3)
        : theme.colorScheme.primary.withValues(alpha: 0.3);

    return Container(
      margin: const EdgeInsets.only(bottom: 12),
      decoration: BoxDecoration(
        border: Border.all(color: borderColor),
        borderRadius: BorderRadius.circular(12),
      ),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Text(
                  providerName,
                  style: theme.textTheme.titleSmall?.copyWith(
                    fontWeight: FontWeight.w600,
                    color: isError ? theme.colorScheme.error : theme.colorScheme.primary,
                  ),
                ),
                const Spacer(),
                if (result.responseTimeMs > 0)
                  _buildResponseTimeBadge(theme, result.responseTimeMs),
                IconButton(
                  icon: const Icon(Icons.copy, size: 16),
                  tooltip: '复制',
                  constraints: const BoxConstraints(minWidth: 32, minHeight: 32),
                  padding: EdgeInsets.zero,
                  onPressed: () {
                    _ffi.setClipboardText(result.translatedText);
                  },
                ),
              ],
            ),
            const Divider(height: 20),
            SelectableText(
              result.translatedText,
              style: theme.textTheme.bodyMedium,
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildResponseTimeBadge(ThemeData theme, int ms) {
    final color = ms < 500
        ? Colors.green
        : ms < 1000
            ? Colors.orange
            : Colors.red;

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.1),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Text(
        '${ms}ms',
        style: theme.textTheme.labelSmall?.copyWith(color: color),
      ),
    );
  }
}