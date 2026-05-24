# Agent 任务：构建一个 Logi Options+ 风格的鼠标配置 UI（GPUI + Rust）

> 复制本文件全文作为 system / initial prompt 投喂给 Claude Code、Cursor、Cline 之类的本地 Agent。
> 中间标注 `[AGENT]` 的指令是给 Agent 的元指令，必须严格遵守。

---

## 0. 角色和工作方式

你是一名熟悉 Rust 和 GPUI 的资深桌面应用工程师。我们要从零构建一个鼠标配置工具的 UI，视觉/交互参考 Logi Options+，但**所有素材、代码、设计资产必须自己产出，绝对不允许从 Logitech 或任何第三方商业产品中提取/复制资源**。

**[AGENT] 工作准则：**

1. **逐阶段交付**：严格按下面 Phase 0 → Phase 7 顺序推进，每个 Phase 都要跑通 `cargo run` 看到可视化结果后再进入下一个。
2. **小步提交**：每个 Phase 完成后做一次 `git commit`，commit message 用 `feat(phaseN): xxx`。
3. **遇到不确定的 GPUI API**：先查 `https://github.com/zed-industries/zed/tree/main/crates/gpui/examples` 里的官方示例，再查 `https://github.com/longbridge/gpui-component/tree/main/examples`，最后才问我。**不要瞎猜 API**。
4. **不要过度工程**：禁止提前抽象。先把硬编码版本跑出来，看到效果，再重构。
5. **每完成一个文件就 `cargo check`**：编译失败立刻修，不要堆积。
6. **占位素材策略**：现阶段没有真实 3D 渲染图，用纯色 SVG / 矩形占位即可，关键是把交互逻辑跑通。

---

## 1. 项目目标

桌面应用，单窗口约 1100×750，分三个主要区域：

```
┌─────────────────────────────────────────────────┐
│  [设备 Carousel：M1 │ M2 │ M3 ...]               │  顶部 80px
├──────────────────────┬──────────────────────────┤
│                      │                          │
│    [鼠标模型]         │   [当前选中按键的         │
│    + 按键热点         │    动作配置面板]          │
│    + 引导线          │                          │
│    + 标签             │   ─ 或 ─                 │
│                      │                          │
│                      │   [DPI / 滚轮 / 手势       │
│                      │    标签切换]              │
│                      │                          │
├──────────────────────┴──────────────────────────┤
│  [设置 / 关于 / 版本号]                          │  底部 50px
└─────────────────────────────────────────────────┘
```

---

## 2. 技术栈

```toml
# Cargo.toml 必备依赖
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed" }
gpui_platform = { git = "https://github.com/zed-industries/zed", features = ["font-kit"] }
gpui-component = { git = "https://github.com/longbridge/gpui-component" }
gpui-component-assets = { git = "https://github.com/longbridge/gpui-component" }
anyhow = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

**不要**自己实现 popover、tooltip、slider、carousel —— 直接用 `gpui-component` 里的。

---

## 3. 文件结构（必须严格遵守）

```
logi-style-ui/
├── Cargo.toml
├── README.md
├── assets/
│   └── mouse/
│       ├── base.svg           # 鼠标底图占位（先用纯色椭圆 SVG）
│       └── glow_*.svg         # 各按键发光叠层占位
├── src/
│   ├── main.rs                # 入口，初始化窗口
│   ├── app.rs                 # 根 view，整体布局
│   ├── theme.rs               # 颜色/尺寸常量
│   ├── state.rs               # 全局状态（当前设备、当前选中按键）
│   ├── mouse_model/
│   │   ├── mod.rs
│   │   ├── view.rs            # MouseModelView 主组件
│   │   ├── hotspots.rs        # 按键热点定义和命中
│   │   ├── leader_lines.rs    # canvas + PathBuilder 引导线
│   │   └── parallax.rs        # 鼠标跟随光标的微旋转
│   ├── components/
│   │   ├── mod.rs
│   │   ├── device_carousel.rs
│   │   ├── dpi_panel.rs       # 含光点预览
│   │   ├── action_popover.rs  # 按键 → 动作选择
│   │   └── gesture_pad.rs     # 手势录制
│   └── data/
│       ├── mod.rs
│       └── mouse_buttons.rs   # ButtonId 枚举 + 热点坐标
```

---

## 4. 阶段任务清单

### Phase 0：项目骨架 (预计 30 分钟)

**目标：** `cargo run` 打开一个 1100×750 的空白窗口，深色背景。

任务：
- `cargo init logi-style-ui`
- 按上面 Cargo.toml 配置依赖
- `src/main.rs` 调用 `gpui_platform::application().run(...)` 打开窗口
- `src/app.rs` 实现 `App` 根 view，背景色 `#1a1a1d`，渲染一行白色文字 "Logi-style UI Bootstrapped"
- `src/theme.rs` 定义颜色常量：`BG_DARK`、`SURFACE`、`ACCENT_BLUE`、`TEXT_PRIMARY`、`TEXT_MUTED`

**验收：** 窗口打开，文字居中显示。截图保存到 `screenshots/phase0.png`。

---

### Phase 1：悬浮高亮（动画练手）(预计 1 小时)

**目标：** 屏幕上画 6 个圆形按钮，hover 时蓝色发光、脉冲，离开恢复。

任务：
- 在 `src/app.rs` 临时铺 6 个 64×64 的圆形 `div`
- 每个用 `.hover()` 改背景色（即时）
- 用 `with_animation` + `Animation::new(...).repeat()` + `ease_in_out` 实现 1.4 秒呼吸式 shadow 脉冲
- 脉冲只在 hover 时启用（条件渲染 with_animation 包裹）

**关键代码片段（参考，不要全抄）：**

```rust
use gpui::{Animation, AnimationExt, BoxShadow, ease_in_out, hsla, point, px};
use std::time::Duration;

fn pulsing_button(idx: usize, hovered: bool) -> impl IntoElement {
    let base = div()
        .id(("btn", idx))
        .size(px(64.))
        .rounded_full()
        .bg(rgb(0x2a2a30))
        .hover(|s| s.bg(rgb(0x3a3a45)));
    
    if hovered {
        base.with_animation(
            "pulse",
            Animation::new(Duration::from_millis(1400))
                .repeat()
                .with_easing(ease_in_out),
            |this, delta| {
                let t = (delta * std::f32::consts::PI).sin();
                this.shadow(vec![BoxShadow {
                    color: hsla(0.6, 0.9, 0.6, 0.3 + t * 0.4),
                    offset: point(px(0.), px(0.)),
                    blur_radius: px(8. + t * 16.),
                    spread_radius: px(2.),
                }])
            },
        ).into_any_element()
    } else {
        base.into_any_element()
    }
}
```

**验收：** 鼠标悬停在按钮上，能看到清晰的蓝色呼吸发光；移开立刻停止。

---

### Phase 2：DPI 面板 + 光点预览 (预计 2 小时)

**目标：** 一个滑块 + 一个预览区，光点以正比于 DPI 的速度水平横移。

任务：
- 新建 `src/components/dpi_panel.rs`
- 上半部分：`gpui_component::slider::Slider`，范围 200..6400
- 下半部分：一个 `400×80` 的深色区域，里面一个 16px 蓝色圆点
- 用 `cx.spawn` + `Timer::after(Duration::from_millis(16))` 驱动光点 x 坐标
- DPI 值变化时立即生效（速度 = `dpi as f32 * 0.5` px/sec）
- 光点走到右边界时从左边重新出现

**关键模式：**

```rust
struct DpiPanel {
    dpi: u32,
    dot_x: f32,
    last_tick: Option<Instant>,
}

impl DpiPanel {
    fn start_loop(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, mut cx| {
            loop {
                Timer::after(Duration::from_millis(16)).await;
                if this.update(&mut cx, |this, cx| {
                    let now = Instant::now();
                    let dt = this.last_tick.map(|t| now.duration_since(t).as_secs_f32()).unwrap_or(0.016);
                    this.last_tick = Some(now);
                    this.dot_x += dt * (this.dpi as f32) * 0.5;
                    if this.dot_x > 400. { this.dot_x = 0.; }
                    cx.notify();
                }).is_err() { break; }
            }
        }).detach();
    }
}
```

**验收：** 拖动滑块，光点速度立刻可见变化。低 DPI 龟速，高 DPI 飞快。

---

### Phase 3：设备 Carousel (预计 1 小时)

**目标：** 顶部一排 3-5 张设备卡片，可水平滑动切换，当前选中卡片有蓝色描边，每张卡片右上角一个连接状态指示灯（脉冲圆点）。

任务：
- 新建 `src/components/device_carousel.rs`
- 直接用 `gpui_component` 里的 Carousel/HorizontalScroll 组件（先 grep 看 examples 里怎么用）
- 数据先硬编码 3 个：MX Master 占位、Lift 占位、M650 占位，各带一个状态：connected / connecting / offline
- 状态指示灯用 Phase 1 的脉冲做法，三种颜色：绿/黄/灰

**验收：** 三张卡片横向排列，点击切换 active 状态有过渡动画，状态灯连接时绿色呼吸、连接中黄色快闪、离线灰色静止。

---

### Phase 4：按键 Popover (预计 2 小时)

**目标：** 屏幕中央放 4 个矩形代表 4 个按键（先用矩形占位，Phase 6 再换真鼠标图），点击任一按键，弹出动作选择 popover。

任务：
- 新建 `src/data/mouse_buttons.rs` 定义：

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ButtonId {
    LeftClick, RightClick, MiddleClick, Back, Forward, DpiToggle,
}

pub struct Hotspot {
    pub id: ButtonId,
    pub bounds: Bounds<Pixels>,  // 相对于鼠标容器
    pub label: &'static str,
}

pub fn default_hotspots() -> Vec<Hotspot> { /* ... */ }
```

- 新建 `src/components/action_popover.rs`，用 `gpui_component::popover::Popover`
- popover 内容：标题 + 一个可选动作列表（"左键单击"、"右键单击"、"复制"、"粘贴"、"截图"、"自定义快捷键"...）
- 点击列表项关闭 popover 并更新 `state.button_bindings`
- 在 `src/state.rs` 用 `Global` 存全局状态：

```rust
pub struct AppState {
    pub current_device: usize,
    pub active_button: Option<ButtonId>,
    pub button_bindings: HashMap<ButtonId, Action>,
    pub dpi: u32,
}
impl Global for AppState {}
```

**验收：** 点击任一按键矩形，popover 在按键右侧弹出（200ms ease_out fade + 4px 上移），选中一个动作后 popover 关闭，按键矩形下方文字更新为所选动作名。

---

### Phase 5：手势录制板 (预计 2 小时)

**目标：** 一个 300×300 的方形区域，按住左键拖动，实时画出蓝色轨迹；松开后判断方向（上/下/左/右/对角），显示在区域下方。

任务：
- 新建 `src/components/gesture_pad.rs`
- 用 `on_mouse_down` / `on_mouse_move` / `on_mouse_up` 收集点序列
- 用 `canvas` 元素 + `PathBuilder::stroke(px(3.))` 画路径
- 松开时根据起点终点向量算 8 方向之一
- **加分项：** 轨迹渐隐 —— 每个点记录 `Instant`，绘制时根据年龄分段降低 alpha

**canvas 用法参考：**

```rust
canvas(
    { let pts = self.points.clone(); move |_, _, _| pts },
    |bounds, pts, window, _cx| {
        if pts.len() < 2 { return; }
        let mut path = PathBuilder::stroke(px(3.));
        path.move_to(pts[0] - bounds.origin);
        for p in &pts[1..] { path.line_to(*p - bounds.origin); }
        if let Ok(p) = path.build() {
            window.paint_path(p, rgb(0x3b82f6));
        }
    },
).size_full()
```

**验收：** 拖动能画出连贯轨迹，松开后下方显示 "→ Right" 之类。

---

### Phase 6：鼠标模型 + 热点（替换 Phase 4 的占位）(预计 3 小时)

**目标：** 把 Phase 4 中央的矩形按键换成真正的鼠标插画 + 透明热点叠层。

任务：
- `assets/mouse/base.svg`：用 SVG 画一个简化鼠标轮廓（一个圆角矩形 + 中间一道滚轮线 + 拇指键缺口），纯色 + 简单渐变，**占位用，不需要写实**
- `assets/mouse/glow_left.svg`、`glow_right.svg`、`glow_middle.svg`：每张只画对应按键的蓝色发光形状，其余透明
- 新建 `src/mouse_model/view.rs`，结构：

```rust
fn render(...) -> impl IntoElement {
    div().relative().size(px(420.), px(560.))
        // 底图
        .child(svg().path("assets/mouse/base.svg").size_full())
        // 发光叠层（按 hovered/active 状态显示）
        .children(ButtonId::iter().map(|btn| {
            let visible = self.highlighted() == Some(btn);
            svg()
                .path(glow_path_for(btn))
                .absolute().inset_0().size_full()
                .opacity(if visible { 1.0 } else { 0.0 })
                // 用 with_animation 做 opacity 过渡
        }))
        // 透明热点
        .children(default_hotspots().iter().map(|h| {
            div()
                .id(("hotspot", h.id))
                .absolute()
                .left(h.bounds.origin.x).top(h.bounds.origin.y)
                .w(h.bounds.size.width).h(h.bounds.size.height)
                .on_hover(...)
                .on_click(...)
        }))
}
```

- 热点的 bounds 坐标在 `default_hotspots()` 里硬编码，对照 SVG 位置调

**验收：** 看到鼠标占位插画，hover 任一按键区域看到对应位置发光，点击后弹出 Phase 4 的 popover。

---

### Phase 7：引导线 + 标签（点睛）(预计 2 小时)

**目标：** 鼠标周围放 4-6 个标签卡片，每个用折线连到对应按键。

任务：
- 新建 `src/mouse_model/leader_lines.rs`
- 标签数据：`Vec<(ButtonId, Point<Pixels>, &'static str)>`，标签的目标位置在鼠标容器外周（左侧 / 右侧）
- 用 `canvas` 画折线：从按键中心点 → 水平延伸 40px → 斜线到标签 anchor
- 当某个按键被 hover/active 时，对应的引导线变蓝加粗，其他变灰

**验收：** 整体效果接近第一版 Logi Options+ 的截图（视觉风格自由发挥，不要复刻）。

---

### Phase 8：微动效收尾 (预计 1 小时)

只在前 7 个 Phase 全部跑通后做：

- **视差**：鼠标容器在 `on_mouse_move` 中根据光标位置做 ±3° 倾斜（用 transform）
- **呼吸**：整个鼠标容器 4 秒周期 ±2px 的 translate_y 循环
- **入场动画**：app 启动时鼠标从 opacity 0 + translate_y 20px 滑入

---

## 5. 关键 GPUI 模式速查（Agent 卡住时回顾）

**动画：**
```rust
element.with_animation(
    "name",                                      // 唯一 ID
    Animation::new(Duration::from_millis(N))
        .repeat()                                // 可选
        .with_easing(ease_in_out),               // 或 linear / bounce
    |this, delta| { /* delta: 0..1 */ this.opacity(delta) },
)
```

**绝对定位叠层：**
```rust
div().relative()
    .child(底图)
    .child(div().absolute().inset_0().size_full().child(叠层))
```

**异步状态更新：**
```rust
cx.spawn(async move |this, mut cx| {
    loop {
        Timer::after(Duration::from_millis(16)).await;
        this.update(&mut cx, |this, cx| { /* mutate */ cx.notify(); }).ok();
    }
}).detach();
```

**全局状态：**
```rust
cx.set_global(AppState { ... });
let state = cx.global::<AppState>();
cx.update_global::<AppState, _>(|s, cx| { s.dpi = 1600; });
```

---

## 6. 常见坑

1. **`with_animation` 必须套在能消费 `delta` 的闭包里**，闭包返回值要是同类型的 element，不要在闭包里改变 element 类型。
2. **canvas 闭包不能捕获 `&mut self`**，要 clone 数据出来。
3. **GPUI 的 `inset_0` + `size_full` 配合 `absolute()` 才能撑满父容器**，单独用 `size_full` 在 absolute 元素里不生效。
4. **gpui-component 的所有组件用前必须先调 `gpui_component::init(cx)`**，放在 app run 的最开头。
5. **`cx.notify()` 是触发重渲染的关键**，状态改了不 notify 不会重绘。
6. **`Timer` 来自 `gpui::Timer`** 不是 tokio 的 Timer。

---

## 7. 资产红线

- **绝对不要**从 Logi Options+ 或任何其他商业应用提取 PNG / Lottie / SVG 资源
- **绝对不要**让 AI 生图工具生成"Logitech 风格"或带具体品牌特征的鼠标
- 所有占位 SVG **自己画**，简单几何形状即可，目标是让交互逻辑可见
- 后期换真实素材时来源必须是：自渲染 / CC0 模型 / 委托设计师

---

## 8. Agent 自检清单

每完成一个 Phase 前，自查：

- [ ] `cargo check` 无错误无 warning
- [ ] `cargo run` 能打开窗口，本 Phase 的功能可视化可交互
- [ ] 新增文件都在规定路径下，没有散落在 `src/` 根目录
- [ ] 写了至少 3 行行内注释解释非显然的代码
- [ ] `git commit` 完成
- [ ] 拍了截图到 `screenshots/phaseN.png`

---

## 9. 开始

从 Phase 0 开始。每完成一个 Phase 输出三件事：

1. 改动的文件清单
2. 截图（或文字描述当前可见状态）
3. 下一步计划

如果遇到 GPUI API 行为不符合预期，**先用最小复现代码（< 50 行）验证你的假设**，再决定改方案。

开始吧。
