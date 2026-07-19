# img2cli Design Specification (Apple Style)

本设计文档为 `img2cli` 的前端界面提供了一套遵循 **Apple 风格（苹果设计语言）** 的设计系统规范。本系统专为暗色模式（Dark Mode）与桌面端配置工具场景进行了深度优化，旨在打造一个如艺术画廊般精致、纯净且无干扰的交互界面。

---

## 1. 核心设计原则 (Core Principles)

1. **界面退后，内容向前 (UI Recedes, Content Speaks)**：去除多余的彩色渐变、阴影和装饰性线条。利用大面积暗色画布（Canvas）、半透明毛玻璃材质（Backdrop Blur）和极细边框（Hairline Border）来构建视觉层次，让用户聚焦在配置和日志内容本身。
2. **单一交互色 (Action Blue Accent)**：全站仅使用一个主交互色——苹果经典的 **Action Blue (#2997ff)**。所有可点击的按钮、链接、激活状态和焦点环都只采用这一种蓝色，给用户最直接的“点击提示”。
3. **负字距 display 标题 (Apple Tight Typography)**：使用 `SF Pro` 字体族（非 Apple 系统自动回退至高品质的 `Inter`），在大字号标题上使用轻微的负字距（Negative Letter-spacing），产生极具苹果质感的紧凑和高级感。
4. **扁平化与微光阴影 (Quiet Elevation)**：除截图选区外，UI 界面本身不使用任何大投影。利用表面颜色（Surface Color）的深浅交替与 1px 细线作为自然分区。

---

## 2. 色彩系统 (Color Tokens)

| Token | 颜色值 | 使用场景 |
| :--- | :--- | :--- |
| `{colors.primary}` | `#2997ff` (Action Blue) | 主交互色、可点击文本、高亮激活状态、焦点边框 |
| `{colors.primary-hover}`| `#40a4ff` | 交互元素悬浮态 |
| `{colors.canvas-dark}` | `#0a0b1e` (Void Black) | 页面最底层背景画布，融入暗色科技感 |
| `{colors.surface-panel}`| `rgba(255, 255, 255, 0.04)` | 主卡片面板背景，支持 `backdrop-blur: 24px` |
| `{colors.surface-sidebar}`| `rgba(255, 255, 255, 0.02)` | 左侧边栏背景，与右侧主视区产生微妙的视觉差 |
| `{colors.border-hairline}`| `rgba(255, 255, 255, 0.08)` | 1px 面板分隔线、输入框边框、表格横线 |
| `{colors.text-primary}` | `#ffffff` | 标题、主要文字 |
| `{colors.text-muted}` | `#94a3b8` (Slate 400) | 次要说明文字、占位符、非激活状态 |
| `{colors.text-disabled}`| `#64748b` (Slate 500) | 禁用文字、页脚微缩法条字样 |

---

## 3. 字体与排版 (Typography System)

非 macOS/iOS 系统开发和运行时，将默认回退至 **Inter** 字体，并通过 `-0.01em` 到 `-0.02em` 的 letter-spacing 重新渲染出苹果紧致风格：

```css
/* 基础字体族声明 */
font-family: "SF Pro Text", "SF Pro Display", "Inter", system-ui, -apple-system, sans-serif;
```

### 排版阶梯 (Typography Ladder)
*   **Hero Display** (24px, Semi-Bold, letter-spacing: `-0.02em`, line-height: 1.15)
    *   *应用场景*：设置页顶部大标题（如 `img2cli Settings`）。
*   **Section Title** (14px, Bold, uppercase, letter-spacing: `0.05em`, text-color: `{colors.text-muted}`)
    *   *应用场景*：卡片区块分组标题（如 `SYSTEM INTEGRATION`）。
*   **Body Copy** (14px, Regular, letter-spacing: `-0.01em`, line-height: 1.5)
    *   *应用场景*：普通表单 Label、输入框文字、说明段落。
*   **Code / Mono** (13px, Regular, font-family: `SF Mono, JetBrains Mono, monospace`)
    *   *应用场景*：快捷键输入框内文字、SSH Host 详情路径、实时日志输出。
*   **Fine Print** (12px, Regular, text-color: `{colors.text-disabled}`)
    *   *应用场景*：版本号展示、输入框下方的微缩提示。

---

## 4. 组件规范 (Component Checklist)

### 4.1 按钮 (Buttons)
*   **Primary Button (胶囊主按钮)**:
    *   样式：`bg-[#2997ff] text-white rounded-full px-5 py-2 text-sm font-medium transition-all`
    *   交互：悬浮时 `bg-[#40a4ff]`，点击时轻微缩放 `scale-95`。
*   **Secondary Button (幽灵次按钮 / Reset 按钮)**:
    *   样式：`bg-[#1d1d1f] text-slate-300 border border-slate-800 rounded-xl px-3 py-1.5 text-xs hover:bg-slate-800 transition-colors`

### 4.2 表单输入框 (Form Inputs)
*   样式：背景 `bg-[#0a0b1e]`，边框 `border-[rgba(255,255,255,0.08)]`，圆角 `rounded-xl`。
*   聚焦状态：边框高亮为 `border-[#2997ff]` 并带有柔和的蓝色内发光 `box-shadow: 0 0 0 1px #2997ff`。

### 4.3 列表与表格 (Tables & Rows)
*   卡片内部表格使用 `border-t border-[rgba(255,255,255,0.08)]` 隔开。
*   操作按钮（如 `Edit`, `Delete`, `Set Default`）应极为克制：平时呈现 `{colors.text-muted}` 色，悬浮时分别高亮为 `{colors.primary}` (蓝色) 和 `text-red-400` (红色)，保持页面色彩纯净。

---

## 5. 截图选区界面设计 (Screen Capture Overlay)

为实现极致的“Snipaste”无瑕感，截图选区设计做如下约束：
*   **选区边框**：2px 纯色实线（使用主交互色 `#2997ff` 或高对比度橙色 `#f97316`）。
*   **选区内部**：完全透明（`background: transparent`），禁止带有任何模糊或半透明着色，保证设计师和程序员读取像素的最佳清晰度。
*   **遮罩外部**：`box-shadow: 0 0 0 9999px rgba(0, 0, 0, 0.4)`，将选区之外的屏幕变暗，高亮突出选区。
