# Rat-Nexus TUI 框架

一个受 GPUI 启发的、功能强大的 TUI（终端用户界面）框架，基于 [Ratatui](https://github.com/ratatui-org/ratatui) 构建。

`Rat-Nexus` 为构建复杂的终端应用程序提供了一种现代化的响应式架构。它具有基于实体的状态管理系统、生命周期钩子以及稳健的事件驱动设计。

## 🚀 特性

- **GPUI 启发式响应性**: 状态通过 `Entity<T>` 管理，它会自动通知订阅者并在更新时触发重新渲染，无需手动调用刷新。
- **分层架构**: 应用程序逻辑、组件渲染和路由之间清晰分离。
- **完善的生命周期管理**: 
  - `on_init`: 组件首次初始化并进入渲染树时调用，适合启动后台任务。
  - `on_exit`: 组件离开当前激活视图前调用，用于清理资源。
  - `on_shutdown`: 应用程序退出前的钩子，确保优雅退出。
- **丰富的渲染上下文**: 组件接收 `Context` 对象，提供所需的一切：绘图区域、应用句柄以及订阅状态的能力。
- **一流的异步支持**: 可以在任何组件中无缝生成（spawn）后台任务，并与应用状态安全交互。

## 🛠 项目结构

```text
.
├── Cargo.toml          # 工作区配置
├── rat-nexus/          # 核心框架库 (Crate)
│   ├── src/
│   │   ├── application.rs   # 应用循环和上下文管理
│   │   ├── component/       # 组件 Trait 定义
│   │   ├── state/           # 实体和响应式状态逻辑
│   │   ├── error.rs         # 基于 Snafu 的错误类型
│   │   └── lib.rs           # 公共接口导出
└── rat-demo/           # 示例应用程序 (Crate)
    ├── src/
    │   ├── pages/           # UI 页面 (菜单、仪表盘/计数器)
    │   ├── model.rs         # 状态数据定义
    │   ├── app.rs           # 根组件/路由器逻辑
    │   └── main.rs          # 程序入口
```


## ⌨️ 快速上手

下面是一个使用 `Rat-Nexus` 构建的最简单计数器示例：

```rust
use rat_nexus::{Application, Component, Context, EventContext, Event, Action, Entity};
use ratatui::widgets::Paragraph;
use crossterm::event::KeyCode;
use std::sync::{Arc, Mutex};

// 1. 定义状态 (State)
struct CounterState { count: i32 }

// 2. 定义组件 (Component)
struct CounterComponent {
    state: Entity<CounterState>,
}

impl Component for CounterComponent {
    fn render(&mut self, frame: &mut ratatui::Frame, cx: &mut Context<Self>) {
        // 订阅状态更新，当 state 改变时自动重绘
        cx.subscribe(&self.state);
        let count = self.state.read(|s| s.count).unwrap_or(0);
        
        frame.render_widget(
            Paragraph::new(format!("Count: {}. Press 'j' to increment.", count)),
            cx.area
        );
    }

    fn handle_event(&mut self, event: Event, _cx: &mut EventContext<Self>) -> Option<Action> {
        if let Event::Key(key) = event {
            if key.code == KeyCode::Char('j') {
                // 更新状态
                self.state.update(|s| s.count += 1);
            }
        }
        None
    }
}

// 3. 运行应用 (Run)
fn main() -> anyhow::Result<()> {
    Application::new().run(|cx| {
        let state = cx.new_entity(CounterState { count: 0 });
        let root = Arc::new(Mutex::new(CounterComponent { state }));
        cx.set_root(root)?;
        Ok(())
    })
}
```

## 🏁 运行演示

### 前置条件

- Rust (最新稳定版)
- Cargo

### 运行演示程序

在项目根目录下通过 Cargo 运行：

```bash
cargo run
```

*(由于已在工作区中将 `rat-demo` 设置为 `default-members`，直接运行即可。)*

### 演示程序操作指南

- `↑/↓ / Enter`: 导航主菜单并进入页面。
- `j / k`: 增加或减少全局计数器的值。
- `w`: 启动一个异步后台“工人”任务（展示异步进度条）。
- `l`: 切换仪表盘布局。
- `c`: 清空事件日志。
- `m`: 返回主菜单。
- `q`: 退出应用程序。

## 💡 核心概念

### 1. 实体与响应式 (Entities & Reactivity)
无需手动刷新，只需将你的状态包装在 `Entity` 中。当你通过 `.update()` 修改状态时，所有订阅了该实体的组件都会自动重新渲染。

```rust
// 更新共享状态
self.state.update(|s| s.counter += 1); // 自动触发组件重绘！
```

### 2. 异步任务 (Async Tasks)
需要时钟或后台文件索引任务？直接使用 `cx.app.spawn`。

```rust
fn on_init(&mut self, cx: &mut Context<Self>) {
    let app = cx.app.clone();
    cx.app.spawn(move |_| async move {
        loop {
            // 执行一些异步工作...
            app.refresh(); // 如有必要，手动触发刷新
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });
}
```

### 3. 组件上下文 (Component Context)
`Context<V>` (以及 `EventContext<V>`) 提供了：
- `cx.area`: 分配给当前组件的 `Rect` 区域。
- `cx.app`: 访问全局应用服务（spawn, refresh 等）。
- `cx.subscribe(entity)`: 监听状态自发更新。
- `cx.cast::<U>()`: 在组件层级中安全地转换上下文类型。

## ⚖️ 开源协议

MIT
