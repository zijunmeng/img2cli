# 参考源码挖掘纪要 — 2026-08-16

> 源: `ref/pkg/` (flameshot / greenshot / ksnip / ShareX / wispterm;spectacle 实为 macOS 窗口平铺工具非 KDE 截图,herdr 未解压)。
> 三路并行挖掘的结论蒸馏,按 img2cli ROADMAP 里程碑归类。**这是 v0.4.0 的实现参考。**

---

## A. Milestone 3 标注编辑器 ← flameshot (ksnip 的引擎库是空 submodule,略)

flameshot 整个引擎没有 QGraphicsScene —— 纯自绘 QWidget + 有序对象列表。五件直接搬:

1. **值对象工具 + `process(ctx, frame)` 契约**: 每个标注 = 不可变数据对象 `{type, points, color, thickness, text}`,带一个 `draw(ctx, frame)` 函数 (pixelate 需要源像素所以传 frame)。编辑器只存 `objects[] + activeTool`,双画布 (原始帧/合成帧)。pointerdown 从原型实例化 → move 更新 → up 校验 `isValid` 后入栈。→ Vue/canvas + store 一比一映射。
2. **增量烘焙 + 快照撤销**: 活动对象画在顶层 canvas,完成后 bake 进合成层;撤销 = `objects[]` 浅拷贝栈 (before/after 每次变更,含移动/改色)。比 flameshot 全量重绘更快: 只从变更索引重画。
3. **马赛克 = 矩形工具 + 相对块宽**: `blockSize = regionSize * 0.5/(strength+1)` (同一滑块任何区域粗细一致);canvas 实现 = 离屏小画布缩小 → `imageSmoothingEnabled=false` 放大。**安全模式**值得抄: 不采样内部像素,只取外圈 1px 边缘 + 固定种子 PRNG 插值 (防 unredacter 攻击还原)。
4. **Alpha 命中测试选中/移动**: 隐形 mask canvas 渲染各对象剪影,点击处 `getImageData(x±3, y±3)` 自顶向下找 —— 自由笔迹像素级选中,零几何代码;点击+拖拽死区(曼哈顿距离>阈值)才启动移动。
5. **箭头几何 + Ctrl 45° 吸附 + 尺寸 UX**: 头宽 `10+t*2` 长 `18+t*4`、短箭头自动缩头、杆缩短防戳出;Ctrl=atan2 取整 45°;滚轮=粗细(数值闪现)、数字键=精确值、右键=光标处取色、双击文本对象重开编辑。

## B. J 窗口识别加固 ← greenshot (Windows 老兵的全套边角)

我们 v0.3.13 的 xcap 朴素方案在 Win10/11 有**真实偏差**,按优先级:

- **P0-1 隐藏窗口过滤**: `DWMWA_CLOAKED` (UWP 挂起/虚拟桌面隐藏) + `ApplicationFrameWindow` 无 `CoreWindow` 子窗 (UWP 空壳)。标题+非最小化挡不住这些幽灵窗。
- **P0-2 矩形源**: 用 `DWMWA_EXTENDED_FRAME_BOUNDS` 而非 GetWindowRect —— 后者含**不可见 7px 阴影边**,我们的蓝框在 Win10/11 上就是偏的;再裁 1px 光晕;最大化窗口跳过 DWM 路径。
- **P0-3 最大化/贴靠溢出**: 最大化窗 rect 超出屏幕一个边框宽 —— 裁 `dwWindowBorders`,最终 rect 必与屏幕求交。Win11 贴靠半屏同此理。
- **P0-4 混合 DPI**: ÷scale 必须用**窗口所在显示器**的 scale (flameshot 按物理尺寸求和;spectacle 按最大面积重叠判定),不能全局一个系数;overlay 也要按显示器,否则副屏悬停框漂移。
- **P0-5 Z 序**: 命中测试要取"最上层包含窗"而非首个/最小 —— xcap 枚举序是否 Z 序需验证;补掉 WS_EX_TOOLWINDOW/Progman/Button/Dwm。
- P1: 矩形冻结 (overlay 打开时快照,点击时不重查——我们已如此 ✓)、边缘点击=整窗/内部下钻 3 层子窗、自窗排除 (已做 ✓)、`-32000` 哨兵坐标显式 IsIconic、光标入图 (M 键开关)。
- P2: 屏外部分透明不涂黑、浏览器标题清洗正则、大图分配上限 (flameshot 撞过 Qt 128MB 默认)。

## C. Milestone 4 贴图 ← ShareX PinToScreenWindow + wispterm

- **ShareX 的 `PinToScreenWindow.axaml.cs` 就是 Snipaste 规格书** (300 行): 窗口精确等尺寸 (Min=Max 锁死系统缩放手柄)、滚轮=缩放 20-500% / Ctrl+滚轮=不透明度 10-100%、右键关、双击缩成点、中键复位、方向键 1px/Shift 10px 微调、**原位钉住** (开在截图位置而非居中)、悬停工具栏、Ctrl+C 复制原图字节。
- **不要每个贴图开一个 Tauri webview** (内存爆炸)。wispterm 证明原生路径可行: `CreateWindowExW` + `SetWindowPos(HWND_TOPMOST)` + GDI `StretchDIBits` 静态贴图,无帧循环,单个贴图成本 ≈ 位图+HWND。Rust 侧用 `tao` 或裸 windows-rs ~200 行。ShareX 的混合模式可借鉴: 首次贴图才懒初始化单一 GUI 面,绝不按贴图数开面。
- **生命周期**: 静态注册表 HashSet + 主线程 post 创建 + Closed 移除 + `CloseAll` 一等热键/托单项。
- **解码前尺寸守卫** (wispterm): PNG 先读尺寸再解码,防恶意大图撑爆内存。

## D. 管线 ← ShareX 任务系统

- **Flags 位掩码任务集** (`AfterCaptureTasks [Flags]`): profile = 任务集合而非固定链;运行时可按条件摘 flag (如非区域截图自动去特效)。我们的 capture→compress→route→SFTP→inject 可演进为 flags。
- **TaskMetadata 捕获时记前台窗口** (标题/进程/rect): 贴图定位、文件命名、"注入到哪个终端"的记录全免费 —— **建议 v0.4.0 顺手做**。
- **`EarlyURLCopyRequested` + `StopUpload()`**: 上传器提前交付结果事件 (配合我们的后台上传快路径) + 协作式取消标志。
- **按数据类型选目的地** (`FTPSelectedImage/Text/File`): "路由"应是目的地配置的属性而非管线阶段 —— 与我们的 RouteResolver 互补。
- 热键双层制: 常规用 RegisterHotKey (wispterm 21 行版,MOD_NOREPEAT,冲突仅记日志);需要按住显示/拦截键时才上 WH_KEYBOARD_LL (ShareX KeyboardHook)。

---

## 落地排序建议 (v0.4.0)

1. **B-P0 全部** (窗口识别正确性,小改动大体验) —— 可先出一个 v0.4.0 前的小版本
2. **A 标注引擎** (五机制,主菜)
3. **C 贴图** (原生窗口 + ShareX 交互规格)
4. D 按需穿插 (TaskMetadata 最先,一行级成本)

*挖掘代理报告全文在会话记录中;本文件是行动向蒸馏。*
