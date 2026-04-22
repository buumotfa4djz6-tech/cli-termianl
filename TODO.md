# TODO - 项目功能实现审查报告

> 审查日期: 2026-04-21
> 项目: cli-terminal (Rust TUI 终端交互增强器)

---

## 一、模块实现状态总览

| 模块 | 状态 | 说明 |
|------|------|------|
| `app` (主事件循环) | ✅ 完整 | 5种模式、子进程管理、渲染全部实现 |
| `config` (配置管理) | ✅ 完整 | YAML加载、Raw→AppConfig转换、默认配置、运行时重载 |
| `input/history` | ✅ 完整 | 历史记录、持久化、搜索 |
| `input/macros` | ✅ 完整 | F键宏绑定 |
| `input/templates` | ✅ 完整 | 模板解析、参数展开、表单面板 |
| `output/highlight` | ✅ 完整 | 基于正则的语法高亮 |
| `output/search` | ✅ 完整 | 正则搜索、匹配导航 |
| `output/collapse` | ✅ 完整 | 可折叠区域管理 |
| `widgets/display` | ✅ 完整 | 输出显示、时间戳、导出、折叠管理 |
| `widgets/input` | ✅ 完整 | Unicode感知输入缓冲区 |

---

## 二、已实现功能清单

### 核心功能
- [x] TUI 界面 (ratatui + crossterm)
- [x] 子进程管理 (stdin/stdout/stderr 管道连接)
- [x] 5种运行模式: Normal / HistorySearch / OutputSearch / TemplateMenu / TemplateForm
- [x] 事件循环 (50ms轮询键盘输入 + 子进程输出 draining)

### 输入模块
- [x] 命令历史记录 (内存 + JSON持久化)
- [x] 历史搜索 (大小写不敏感模糊匹配)
- [x] F2-F12 宏快捷键
- [x] 命令模板菜单 (F1唤起)
- [x] 参数表单面板 (Tab导航、Ctrl+D下拉选项、默认值)
- [x] Unicode 输入支持 (grapheme cluster)

### 输出模块
- [x] 正则语法高亮 (多规则, 首次匹配优先)
- [x] 输出搜索 (正则, n/N 导航)
- [x] 可折叠输出区域 (自动创建, C键切换)
- [x] 时间戳 (可配置格式)
- [x] 输出导出 (Ctrl+E, 保存到配置文件目录)

### 配置系统
- [x] YAML 配置文件 (~/.config/cli-terminal/config.yaml)
- [x] 内嵌默认配置 (defaults.yaml)
- [x] 运行时配置重载 (Ctrl+L)
- [x] 颜色反序列化 (名称 → ratatui::Color)
- [x] 命令自动补全 (Tab键, 基于commands配置)
- [x] 配置容错 (文件不存在/格式错误时回退到默认)

### CLI
- [x] 可选目标程序参数 (`cargo run -- <program>`)
- [x] 帮助信息 (clap derive)

---

## 三、与 CLAUDE.md 描述的差异

| CLAUDE.md 描述 | 实际实现 | 差异 |
|---|---|---|
| "ctrl+E to export output" | ✅ 已实现 | 无差异 |
| "Ctrl+R for history search" | ✅ 已实现 | 无差异 |
| "`/` for output search" | ✅ 已实现 | 无差异 |
| "F1 for template menu" | ✅ 已实现 | 无差异 |
| "F2-F12 for macros" | ✅ 已实现 | 无差异 |
| "Ctrl+E to export output" | ✅ 已实现 | 无差异 |
| "Ctrl+L to reload config" | ✅ 已实现 | 无差异 |
| "n/N for next/previous match" | ✅ 已实现 | 无差异 |
| "suspend terminal for parameter prompting" | ⚠️ 部分实现 | 使用表单面板替代了"挂起终端"方式, 功能等效 |

---

## 四、代码质量问题与潜在 Bug

### 中等优先级
1. **reload_config 丢失已有显示内容** (`app.rs:214-221`): `reload_config()` 创建全新的 `DisplayWidget`, 导致之前的输出内容丢失。应只更新时间戳设置而不是替换整个 widget。

2. **渲染时输出滚动逻辑** (`app.rs:970`): `scroll()` 使用 `self.display.len()` 计算偏移, 但折叠后实际显示行数更少, 导致滚动可能过头。

3. **history_query 字段复用** (`app.rs`): `history_query` 字符串在 HistorySearch 和 OutputSearch 两种模式下复用, 语义不清晰, 容易造成状态污染。

4. **搜索模式 Backspace 处理** (`app.rs:468-477`): 非 ASCII 字符时 `pop()` 会 panic, 应该使用 `pop()` 前的 UTF-8 检查或改用 `truncate`。

5. **子进程退出后继续空转** (`app.rs:324-333`): `drain_child_output` 在收到退出信号后 `break`, 但后续 event loop 循环中 `try_recv` 仍可能被调用。channel 不会被关闭, 不会 panic, 但逻辑上应该标记已退出。

### 低优先级
6. **颜色仅支持基础色** (`manager.rs:132-144`): 不支持 RGB 颜色或 256 色, 仅支持 16 种预定义颜色名称。

7. **无单元测试** (`src/` 下无 `#[cfg(test)]` 模块): 除 `lib.rs` 中的 defaults YAML 测试外, 各模块 (HistoryManager, MacroManager, Template, InputBuffer, SearchState, CollapseManager) 均无单元测试。examples/test_target.rs 有测试但不属于主库。

8. **InputBuffer 批量操作缺失**: 不支持 Ctrl+W (删除单词)、Ctrl+U (删除到行首)、Ctrl+K (删除到行尾) 等常见 readline 快捷键。

---

## 五、测试覆盖情况

| 测试文件 | 测试数量 | 覆盖内容 |
|---|---|---|
| `lib.rs::test_defaults_yaml` | 3 | defaults.yaml 解析、序列化往返、命令模板存在性 |
| `examples/test_target.rs` | 10 | 测试目标的命令分发逻辑 (不属于主库) |
| **主库各模块** | **0** | ❌ 无单元测试 |

---

## 六、建议改进项

### 功能增强
- [ ] 添加 Ctrl+W / Ctrl+U / Ctrl+K 等常见编辑快捷键
- [ ] 支持 RGB / 256 色高亮配置
- [ ] 添加配置验证错误的具体位置提示
- [ ] 支持多关键词历史搜索
- [ ] 输出搜索支持高亮所有匹配而非仅当前匹配
- [ ] 添加鼠标事件支持 (点击折叠、滚动)

### 代码质量
- [ ] 为 HistoryManager, InputBuffer, Template, SearchState 添加单元测试
- [ ] 修复 reload_config 的显示内容丢失问题
- [ ] 修复搜索模式 Backspace 的 UTF-8 安全性
- [ ] 重构 history_query 字段, 分离不同模式的查询字符串
- [ ] 考虑使用 `arc-swap` 或类似机制实现无锁配置热更新
