# SuDoKube UI 重构规划

## 背景

当前 SuDoKube 使用 ratatui 构建 TUI 界面，参考了 ratatui 官方 examples。为了提升用户体验并简化代码，计划进行 UI 重构。

## Ratatui Examples 参考

### 关键功能
- **Popup**: 使用 `Clear` widget + `Block::bordered` 创建弹窗，可居中显示
- **Scrollbar**: `Scrollbar` widget + `ScrollbarState` 管理滚动状态
- **Flex**: `Flex` 枚举 + `Constraint::Fill` 实现弹性布局，撑满可用空间
- **Custom Widget**: 实现 `Widget` trait，直接操作 `Buffer`
- **User Input**: 表单输入组件

### 布局策略
- 使用 `Layout::flex(Flex::Stretch)` 撑满画面
- 嵌套 `Layout` 实现复杂布局
- 响应式设计，适配不同终端尺寸

## 重构目标

### 1. 设置界面重构
**现状**: 独立跳转 `AppScreen::Settings`，独占全屏

**目标**: 
- 使用 `Popup` 模式，不离开游戏/菜单
- 添加 `Scrollbar` 支持大量设置项滚动
- 支持 `←/→` 切换选项，`Esc` 关闭

**参考**: `popup` + `scrollbar` examples

### 2. 导入对局界面重构
**现状**: 独立跳转 `AppScreen::ImportInput`，全屏输入框

**目标**:
- 改为内联 `Popup` 样式
- 使用 `Paragraph` 模拟输入框
- 支持粘贴事件

### 3. 游戏内部按钮重构
**现状**: 使用 `Block::bordered` + `Paragraph` 渲染按钮

**目标**:
- 创建自定义 `Button` widget
- 支持鼠标点击交互
- 实现 hover 效果

**参考**: `custom-widget` example

### 4. 整体布局 Flex 化
**现状**: 
- 使用固定 `Constraint::Length` 布局
- 左右列自适应但不够灵活

**目标**:
- 核心区域使用 `Constraint::Fill` 撑满
- 游戏区域使用 `Flex::Stretch` 弹性布局
- 边缘面板固定宽度

### 5. 数独面板响应式设计
**现状**: 固定宽度布局，小屏幕显示不友好

**目标**:
- 检测终端尺寸是否足够显示完整数独
- 尺寸不足时在信息栏提示 "按 M 切换模式"
- 支持滚动浏览

### 6. 模式简化
**现状**: `Standard` + `Monospace` (等距)

**目标**: 改为三种模式
- **Scrollbar**: 使用滚动条显示完整数独
- **精简**: 紧凑布局，最小空间占用
- **标准**: 当前标准显示

### 7. 设置项整理
**待移除设置**:
- `standard_cell_width` (精简模式下忽略)
- `cube_scale`, `cube_width`, `cube_height`, `cube_aspect` (精简模式隐藏 cube)
- 部分颜色设置可简化为预设主题

## 实施计划

### Phase 1: 基础设施 (预计 1-2 天)
- [ ] 创建自定义 `Button` widget
- [ ] 创建自定义 `Popup` widget
- [ ] 创建 `ScrollableList` widget
- [ ] 建立 UI 组件库结构

### Phase 2: 布局重构 (预计 2-3 天)
- [ ] 重构主布局为 Flex 布局
- [ ] 实现响应式尺寸检测
- [ ] 添加模式切换逻辑
- [ ] 更新 `GameLayout` 结构体

### Phase 3: 界面迁移 (预计 2-3 天)
- [ ] 设置界面改为 Popup
- [ ] 导入界面改为 Popup
- [ ] 游戏按钮改为自定义 Widget
- [ ] 更新输入事件处理

### Phase 4: 细节优化 (预计 1-2 天)
- [ ] 添加动画效果 (可选)
- [ ] 优化错误提示
- [ ] 更新 i18n 翻译
- [ ] 测试不同终端尺寸

## 文件变更清单

### cli/src/
- `render.rs`: 主要重构对象
  - 新增 `Button` widget 实现
  - 新增 `Popup` widget 实现
  - 新增 `ScrollableList` widget 实现
  - 重构 `compute_game_layout_from_rect` 函数
  - 重构 `draw_settings` 为 Popup 模式
  - 重构 `draw_import_input` 为 Popup 模式

- `input.rs`: 更新事件处理
  - 移除独立的 Settings/ImportInput 事件处理
  - 添加 Popup 模式的事件处理
  - 更新游戏按钮点击检测

- `main.rs`: 更新应用状态
  - 简化 `AppScreen` 枚举
  - 移除冗余的设置状态
  - 添加 `Popup` 相关状态

- `i18n.rs`: 更新翻译
  - 添加新模式名称
  - 更新按钮标签

## 测试计划

### 单元测试
- [ ] 自定义 Widget 渲染测试
- [ ] 布局计算测试
- [ ] 事件处理测试

### 集成测试
- [ ] 不同终端尺寸测试 (80x24, 120x40, 全屏)
- [ ] 所有界面流程测试
- [ ] 键盘/鼠标交互测试

### 手动测试
- [ ] 正常游戏流程
- [ ] 设置修改流程
- [ ] 导入导出流程
- [ ] 小屏幕适配

## 风险评估

### 高风险
- **布局重构**: 可能影响现有布局兼容性
- **响应式设计**: 不同终端表现可能不一致

### 中风险
- **自定义 Widget**: 需要测试多种终端背景色支持
- **事件处理**: 鼠标/键盘事件可能冲突

### 低风险
- **i18n**: 字符串变更，用户习惯可能需要适应

## 注意事项

1. **向后兼容**: 确保现有快捷键仍可用
2. **渐进式**: 分阶段提交，便于回滚
3. **可访问性**: 考虑色盲用户，优化颜色对比
4. **性能**: 避免频繁重绘，保持 60fps

## 预期效果

- 界面更紧凑，节省空间
- 设置更便捷，无需跳转
- 小屏幕友好，自动提示
- 代码更模块化，易维护
