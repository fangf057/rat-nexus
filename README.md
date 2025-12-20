# Rat-Nexus TUI 框架

一个受 GPUI 启发的、功能强大的 TUI（终端用户界面）框架，基于 [Ratatui](https://github.com/ratatui-org/ratatui) 构建。

`Rat-Nexus` 为构建复杂的终端应用程序提供了一种现代化的响应式架构。它具有基于实体的状态管理系统、完善的生命周期钩子、可取消的异步任务以及类型安全的路由系统。

![demo](./asserts/bkg.png)

## 🚀 特性

- **GPUI 启发式响应性**: 状态通过 `Entity<T>` 管理，自动通知订阅者并触发重新渲染。
- **完善的生命周期管理**:
  - `on_mount`: 组件首次挂载时调用一次，适合启动后台任务。
  - `on_enter`: 每次进入组件视图时调用（导航进入）。
  - `on_exit`: 离开组件视图时调用，用于清理资源。
  - `on_shutdown`: 应用程序退出前的钩子。
- **可取消的异步任务**: `TaskHandle` 和 `TaskTracker` 支持任务生命周期管理，组件退出时自动取消任务。
- **类型安全路由**: `Router<R>` 泛型路由器 + `define_routes!` 宏实现编译时路由检查。
- **简化的状态订阅**: `cx.watch()` 方法一行代码完成订阅和读取。
- **一流的异步支持**: 在任何组件中无缝生成后台任务，并与应用状态安全交互。

## 🛠 项目结构

```text
.
├── Cargo.toml          # 工作区配置
├── rat-nexus/          # 核心框架库
│   ├── src/
│   │   ├── application.rs   # 应用循环、Context、AppContext
│   │   ├── component/       # Component trait 定义
│   │   ├── state/           # Entity 响应式状态
│   │   ├── router/          # Router 和 define_routes! 宏
│   │   ├── task.rs          # TaskHandle、TaskTracker
│   │   ├── error.rs         # 错误类型
│   │   └── lib.rs           # 公共接口导出
└── rat-demo/           # 示例应用程序
    ├── src/
    │   ├── pages/           # UI 页面 (菜单、计数器、贪吃蛇)
    │   ├── model.rs         # 状态数据定义
    │   ├── app.rs           # 根组件/路由逻辑
    │   └── main.rs          # 程序入口
```

## ⌨️ 快速上手

### 最简计数器示例

```rust
use crossterm::event::KeyCode;
use rat_nexus::{Action, Application, Component, Context, Entity, Event, EventContext};
use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Color, Stylize},
    widgets::{Block, BorderType, Paragraph},
};
use std::sync::{Arc, Mutex};

struct CounterState {
    count: i32,
}

struct CounterComponent {
    state: Entity<CounterState>,
}

impl Component for CounterComponent {
    fn render(&mut self, frame: &mut ratatui::Frame, cx: &mut Context<Self>) {
        // 使用 watch 一行完成订阅+读取
        let count = cx.watch(&self.state, |s| s.count).unwrap_or(0);

        let area = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(5),
            Constraint::Fill(1),
        ])
        .split(cx.area)[1];

        let area = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(40),
            Constraint::Fill(1),
        ])
        .split(area)[1];

        let color = if count >= 0 { Color::Yellow } else { Color::Blue };

        let text = vec![
            ratatui::text::Line::from(vec!["Value: ".into(), format!("{count}").bold().fg(color)]),
            "".into(),
            ratatui::text::Line::from(" [j]↑  [k]↓  [q]Quit ").dim(),
        ];

        frame.render_widget(
            Paragraph::new(text).alignment(Alignment::Center).block(
                Block::bordered()
                    .title(" Counter ")
                    .title_alignment(Alignment::Center)
                    .border_type(BorderType::Rounded),
            ),
            area,
        );
    }

    fn handle_event(&mut self, event: Event, _cx: &mut EventContext<Self>) -> Option<Action> {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char('j') => { let _ = self.state.update(|s| s.count += 1); }
                KeyCode::Char('k') => { let _ = self.state.update(|s| s.count -= 1); }
                KeyCode::Char('q') => return Some(Action::Quit),
                _ => {}
            },
            _ => {}
        }
        None
    }
}

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

### 运行

```bash
cargo run
```

### 操作指南

- `↑/↓ / Enter`: 导航菜单并进入页面
- `j / k`: 增加或减少计数器
- `w`: 启动异步后台任务
- `l`: 切换布局
- `c`: 清空日志
- `m`: 返回主菜单
- `q`: 退出

**贪吃蛇游戏**:
- `←↑↓→` 或 `wasd`: 控制方向
- `Space`: 暂停/继续
- `r`: 重新开始

## 💡 核心概念

### 1. 生命周期钩子

```rust
impl Component for MyPage {
    /// 首次挂载时调用一次 - 适合启动后台任务
    fn on_mount(&mut self, cx: &mut Context<Self>) {
        let handle = cx.spawn_task(|_| async move {
            loop {
                // 后台工作...
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });
        self.tasks.track(handle);
    }

    /// 每次导航进入时调用
    fn on_enter(&mut self, cx: &mut Context<Self>) {
        // 重置临时状态等
    }

    /// 离开视图时调用
    fn on_exit(&mut self, cx: &mut Context<Self>) {
        // 取消所有后台任务
        self.tasks.abort_all();
    }

    /// 应用关闭时调用
    fn on_shutdown(&mut self, cx: &mut Context<Self>) {
        // 最终清理
    }
}
```

### 2. 可取消的异步任务

```rust
use rat_nexus::{TaskHandle, TaskTracker};

struct MyComponent {
    tasks: TaskTracker,  // 自动管理多个任务
}

impl Component for MyComponent {
    fn on_mount(&mut self, cx: &mut Context<Self>) {
        // spawn_task 返回可取消的 handle
        let handle = cx.spawn_task(|app| async move {
            loop {
                // 异步工作...
                app.refresh();
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });

        // 追踪任务，组件退出时自动取消
        self.tasks.track(handle);
    }

    fn on_exit(&mut self, _cx: &mut Context<Self>) {
        self.tasks.abort_all();  // 取消所有任务
    }
}

// TaskTracker 实现了 Drop，析构时自动 abort_all()
```

### 3. 实体与响应式

```rust
// 创建实体
let state = cx.new_entity(MyState::default());

// 更新状态 - 自动通知订阅者
self.state.update(|s| s.counter += 1);

// 读取状态
let value = self.state.read(|s| s.counter).unwrap();

// 订阅+读取一体化
let value = cx.watch(&self.state, |s| s.counter).unwrap();
```

### 4. 类型安全路由

```rust
use rat_nexus::{define_routes, Router};

// 使用宏定义路由枚举
define_routes! {
    Menu,
    Counter,
    Settings,
    Snake,
}

// 创建路由器
let mut router = Router::new(Route::Menu);

// 导航
router.navigate(Route::Counter);  // 编译时类型检查！

// 返回
if router.can_go_back() {
    router.go_back();
}

// 获取当前路由
match router.current() {
    Route::Menu => { /* ... */ }
    Route::Counter => { /* ... */ }
    // ...
}
```

### 5. 组件上下文

`Context<V>` 提供：

| 方法/字段 | 说明 |
|-----------|------|
| `cx.area` | 组件渲染区域 `Rect` |
| `cx.app` | 应用上下文 `AppContext` |
| `cx.subscribe(entity)` | 订阅实体变更 |
| `cx.watch(entity, f)` | 订阅+读取一体化 |
| `cx.spawn(f)` | 生成后台任务 |
| `cx.spawn_task(f)` | 生成可取消任务，返回 `TaskHandle` |
| `cx.notify()` | 手动触发重渲染 |
| `cx.cast::<U>()` | 转换上下文类型 |

## 📦 API 速览

```rust
// 核心导出
pub use rat_nexus::{
    // 应用
    Application, AppContext, Context, EventContext,
    // 组件
    Component, Event, Action, AnyComponent,
    // 状态
    Entity, WeakEntity,
    // 路由
    Router, Route, define_routes,
    // 任务
    TaskHandle, TaskTracker,
    // 错误
    Error, Result,
};
```

## 🔧 与原 on_init 的区别

| 旧 API | 新 API | 说明 |
|--------|--------|------|
| `on_init` (每次导航都调用) | `on_mount` (仅首次) | 防止任务重复 spawn |
| 需要 `initialized` 标志 | 不需要 | 框架保证只调用一次 |
| `cx.spawn` (无法取消) | `cx.spawn_task` → `TaskHandle` | 支持任务取消 |
| 手动管理任务生命周期 | `TaskTracker` 自动管理 | Drop 时自动取消 |

## ⚖️ 开源协议

MIT
