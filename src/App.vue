<template>
  <!-- Region-capture overlay (screenshot hotkey opens index.html?capture=1) -->
  <div v-if="captureMode" class="fixed inset-0 z-[9999] cursor-crosshair select-none"
       @mousedown="capMouseDown" @mousemove="capMouseMove" @mouseup="capMouseUp"
       @contextmenu.prevent="cancelRect">
    <img v-if="capturedImageSrc" :src="capturedImageSrc" class="absolute inset-0 w-full h-full object-cover pointer-events-none" />
    <!-- Annotation canvas (v0.4.2): full-overlay, pointer-transparent, clipped
         to the selection; annotations live in overlay coordinates so they stay
         glued to the frozen frame when the selection moves. -->
    <canvas ref="annotCanvas" class="absolute inset-0 w-full h-full pointer-events-none z-[10001]"></canvas>
    <!-- In-progress text editor (text tool) -->
    <textarea v-if="editingText" ref="textEditor" v-model="editingText.value" autofocus spellcheck="false"
      class="absolute z-[10005] bg-transparent outline-none border border-dashed border-white/60 px-0.5 py-0 resize-none overflow-hidden whitespace-pre"
      :style="{ left: editingText.x + 'px', top: editingText.y + 'px', color: toolColor, fontSize: textFontSize + 'px', lineHeight: textFontSize + 'px', minWidth: '120px', height: (textFontSize + 8) + 'px', fontWeight: 600 }"
      @mousedown.stop.prevent @keydown.stop="textKeydown" @blur="commitText"></textarea>
    <!-- Key guide, Snipaste-style bottom-left panel (one key per line) -->
    <div v-if="config.capture_show_hints && !editingText" class="absolute bottom-5 left-5 bg-black/70 text-white/90 text-[11px] leading-relaxed px-3.5 py-2.5 rounded-lg pointer-events-none shadow-lg z-[10006] font-mono space-y-0.5">
      <div v-for="(line, i) in hintLines" :key="i">{{ line }}</div>
    </div>
    <!-- Auto-detected window under the cursor (6-J): outline + size label -->
    <div v-if="hoverRect && !hasRect" :style="hoverStyle" class="absolute pointer-events-none z-[10000]">
      <span class="absolute -top-6 left-0 bg-[#2997ff] text-white text-[11px] px-1.5 py-0.5 rounded-md font-mono whitespace-nowrap shadow-lg">{{ Math.round(hoverRect.w) }} × {{ Math.round(hoverRect.h) }}</span>
    </div>
    <div v-if="hasRect" :style="[rectStyle, selBorderStyle]" @mousedown.stop="rectMouseDown"
         :class="['absolute border-solid border-[#2997ff] box-border z-[10000]', activeTool === 'select' && confirmed ? 'cursor-move' : 'cursor-crosshair']">
      <div v-for="hd in handles" :key="hd" :style="handleStyle(hd)" :class="handleCursorClass(hd)"
           @mousedown.stop.prevent="startResize(hd, $event)"
           class="absolute w-2.5 h-2.5 bg-white border border-[#2997ff] rounded-sm shadow"></div>
      <!-- Toolbar: annotation tools + actions (Snipaste-style icon buttons with
           hover tooltips). -->
      <div :style="toolbarStyle" @mousedown.stop
           class="absolute flex items-center gap-0.5 bg-[#1a1a1a]/95 backdrop-blur rounded-lg px-1.5 py-1 text-white shadow-xl">
        <span class="px-1 tabular-nums text-white/80 text-xs">{{ Math.round(rect.w) }} × {{ Math.round(rect.h) }}</span>
        <span class="w-px h-4 bg-white/20 mx-0.5"></span>
        <button v-for="tl in toolButtons" :key="tl.id" @click.stop="setTool(tl.id)"
                :title="tl.tip" :aria-label="tl.tip"
                :class="['w-7 h-7 rounded-md text-sm flex items-center justify-center transition-colors', activeTool === tl.id ? 'bg-[#2997ff] text-white' : 'hover:bg-white/15 text-white/90']">{{ tl.icon }}</button>
        <span class="w-px h-4 bg-white/20 mx-0.5"></span>
        <button @click.stop="undoAnnot" :title="t('Undo') + ' (Ctrl+Z)'" class="w-7 h-7 rounded-md hover:bg-white/15 text-white/90 text-sm flex items-center justify-center">↶</button>
        <button @click.stop="redoAnnot" :title="t('Redo') + ' (Ctrl+Y)'" class="w-7 h-7 rounded-md hover:bg-white/15 text-white/90 text-sm flex items-center justify-center">↷</button>
        <button @click.stop="colorMenu = !colorMenu" :title="t('Color')"
                class="w-7 h-7 rounded-md hover:bg-white/15 flex items-center justify-center">
          <span class="w-3.5 h-3.5 rounded-full border border-white/70" :style="{ backgroundColor: toolColor }"></span>
        </button>
        <div v-if="colorMenu" class="absolute top-8 left-0 bg-[#1a1a1a]/95 backdrop-blur rounded-lg p-1.5 flex gap-1 shadow-xl" @mousedown.stop>
          <button v-for="c in palette" :key="c" @click.stop="toolColor = c; colorMenu = false"
                  class="w-5 h-5 rounded-full border border-white/40 hover:scale-110 transition-transform" :style="{ backgroundColor: c }"></button>
        </div>
        <div class="flex items-center gap-0.5 px-0.5" :title="t('Thickness')">
          <button @click.stop="toolSize = Math.max(1, toolSize - 1)" class="w-5 h-7 rounded hover:bg-white/15 text-xs">−</button>
          <span class="text-[11px] w-4 text-center tabular-nums">{{ toolSize }}</span>
          <button @click.stop="toolSize = Math.min(8, toolSize + 1)" class="w-5 h-7 rounded hover:bg-white/15 text-xs">+</button>
        </div>
        <span class="w-px h-4 bg-white/20 mx-0.5"></span>
        <button @click.stop="actionPin" :title="t('Pin to screen')" class="w-7 h-7 rounded-md hover:bg-white/15 text-sm flex items-center justify-center">📌</button>
        <button @click.stop="actionSave" :title="t('Save to file')" class="w-7 h-7 rounded-md hover:bg-white/15 text-sm flex items-center justify-center">💾</button>
        <button @click.stop="actionCopy" :title="t('Copy image to clipboard') + ' (Ctrl+C)'" class="w-7 h-7 rounded-md hover:bg-white/15 text-sm flex items-center justify-center">📋</button>
        <button @click.stop="confirmSelection" :title="t('Confirm selection (then move/annotate)')"
                :class="['px-2 h-7 rounded-md text-xs font-medium flex items-center', confirmed ? 'bg-white/20 text-white/60' : 'bg-white/15 hover:bg-white/25 text-white']">✓</button>
        <button @click.stop="confirmRect" :title="t('Upload + inject')" class="px-2 h-7 rounded-md bg-[#2997ff] hover:brightness-110 text-xs font-medium flex items-center">⬆</button>
        <button @click.stop="cancelRect" :title="t('Cancel')" class="w-6 h-7 rounded-md hover:bg-white/15 text-xs">✕</button>
      </div>
    </div>
  </div>
  <!-- Pin-to-screen window (index.html?pin=ID) -->
  <div v-else-if="pinMode" class="fixed inset-0 overflow-hidden select-none bg-transparent"
       @contextmenu.prevent="pinMenu = !pinMenu" @dblclick="closePin" @wheel.prevent="pinZoom" @mousedown="pinMenu = false">
    <img v-if="pinImg" :src="pinImg" draggable="false"
         class="w-full h-full object-fill cursor-move" @mousedown.stop="startPinDrag" />
    <div v-if="pinMenu" class="absolute top-1 left-1 z-50 bg-[#1a1a1a]/95 backdrop-blur rounded-lg py-1 text-xs text-white shadow-xl min-w-[140px]"
         @mousedown.stop @contextmenu.prevent>
      <button class="w-full text-left px-3 py-1.5 hover:bg-white/15" @click="pinCopy">📋 {{ t('Copy image to clipboard') }}</button>
      <button class="w-full text-left px-3 py-1.5 hover:bg-white/15" @click="pinSaveAs">💾 {{ t('Save to file') }}</button>
      <button class="w-full text-left px-3 py-1.5 hover:bg-white/15 text-red-300" @click="closePin">🗑 {{ t('Destroy pin') }}</button>
    </div>
  </div>
  <div
    v-else 
    :style="{
      '--bg-app': currentTheme.bgApp,
      '--bg-sidebar': currentTheme.bgSidebar,
      '--bg-card': currentTheme.bgCard,
      '--color-border': currentTheme.colorBorder,
      '--color-accent': currentTheme.colorAccent,
      '--color-accent-hover': currentTheme.colorAccentHover,
      '--color-accent-dim': currentTheme.colorAccentDim,
      '--color-text-primary': currentTheme.textPrimary,
      '--color-text-secondary': currentTheme.textSecondary,
      '--bg-input': currentTheme.bgInput,
      '--color-input-border': currentTheme.colorInputBorder,
      '--bg-toggle': currentTheme.bgToggle,
      '--color-toggle-knob': currentTheme.colorToggleKnob,
      '--bg-button': currentTheme.bgButton,
      '--bg-button-hover': currentTheme.bgButtonHover
    }"
    class="relative flex h-screen text-[var(--color-text-primary)] font-sans overflow-hidden bg-[var(--bg-app)]"
  >
    <!-- Ambient background glows (give the frosted glass something to blur) -->
    <div class="pointer-events-none absolute inset-0 z-0 overflow-hidden">
      <div class="absolute -bottom-32 -left-24 w-[30rem] h-[30rem] rounded-full bg-[var(--color-accent)]/[0.04] blur-[120px]"></div>
      <div class="absolute top-1/4 -right-24 w-[28rem] h-[28rem] rounded-full bg-fuchsia-600/[0.02] blur-[120px]"></div>
      <div class="absolute -bottom-32 left-1/3 w-[26rem] h-[26rem] rounded-full bg-indigo-600/[0.02] blur-[120px]"></div>
    </div>
    <!-- Sidebar -->
    <div class="relative z-10 w-64 bg-[var(--bg-sidebar)] backdrop-blur-2xl border-r border-[var(--color-border)] flex flex-col shrink-0">
      <div>
        <div class="p-6 border-b border-[var(--color-border)] flex items-center gap-3">
          <img src="./assets/logo.png" class="w-8 h-8 rounded-lg shadow-lg shadow-[var(--color-accent)]/10 object-contain" alt="img2cli Logo" />
          <div>
            <h1 class="text-lg font-bold text-[var(--color-text-primary)] tracking-tight">img2cli</h1>
            <p class="text-xs text-[var(--color-text-secondary)]">{{ t('Settings') }} v{{ APP_VERSION }}</p>
          </div>
        </div>

        <nav class="p-4 space-y-1">
          <button 
            @click="activeTab = 'general'"
            :class="['w-full flex items-center gap-3 px-4 py-3 rounded-xl transition-all duration-200 font-medium text-sm', activeTab === 'general' ? 'bg-[var(--color-accent)] text-white shadow-sm shadow-[var(--color-accent)]/15' : 'text-[var(--color-text-secondary)] hover:bg-white/[0.02] hover:text-[var(--color-text-primary)]']"
          >
            <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4" />
            </svg>
            {{ t('General Settings') }}
          </button>

          <button 
            @click="activeTab = 'hosts'"
            :class="['w-full flex items-center gap-3 px-4 py-3 rounded-xl transition-all duration-200 font-medium text-sm', activeTab === 'hosts' ? 'bg-[var(--color-accent)] text-white shadow-sm shadow-[var(--color-accent)]/15' : 'text-[var(--color-text-secondary)] hover:bg-white/[0.02] hover:text-[var(--color-text-primary)]']"
          >
            <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
            </svg>
            {{ t('Hosts & Targets') }}
          </button>

          <button 
            @click="activeTab = 'logs'"
            :class="['w-full flex items-center gap-3 px-4 py-3 rounded-xl transition-all duration-200 font-medium text-sm', activeTab === 'logs' ? 'bg-[var(--color-accent)] text-white shadow-sm shadow-[var(--color-accent)]/15' : 'text-[var(--color-text-secondary)] hover:bg-white/[0.02] hover:text-[var(--color-text-primary)]']"
          >
            <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
            </svg>
            {{ t('System Logs') }}
          </button>
        </nav>
      </div>
    </div>

    <!-- Main Content -->
    <div class="relative z-10 flex-1 flex flex-col min-w-0 overflow-y-auto">
      <main class="flex-1 p-8 max-w-4xl w-full mx-auto space-y-6">
        
        <!-- General Settings Tab -->
        <div v-if="activeTab === 'general'" class="space-y-6">
          <div class="flex justify-between items-center">
            <div>
              <h2 class="text-2xl font-bold tracking-tight text-[var(--color-text-primary)]">{{ t('General Settings') }}</h2>
              <p class="text-sm text-[var(--color-text-secondary)]">{{ t('Configure global screenshot format, hotkeys, and injection preferences.') }}</p>
            </div>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <!-- Left Card -->
            <div class="bg-[var(--bg-card)] backdrop-blur-2xl border border-[var(--color-border)] rounded-2xl p-6 space-y-4 shadow-[0_8px_32px_rgba(0,0,0,0.37)]">
              <h3 class="text-sm font-semibold uppercase text-[var(--color-text-secondary)] tracking-wider">{{ t('Image Config') }}</h3>
              
              <div>
                <label class="block text-xs font-semibold text-[var(--color-text-secondary)] mb-1">{{ t('Output Format') }}</label>
                <select v-model="config.output_format" class="w-full bg-[var(--bg-input)] border border-[var(--color-input-border)] rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-accent)] text-[var(--color-text-primary)]">
                  <option value="markdown">{{ t('Markdown (![image](path))') }}</option>
                  <option value="html">HTML (&lt;img src="path" /&gt;)</option>
                  <option value="raw">{{ t('Raw Path') }}</option>
                  <option value="base64">{{ t('Inline Base64 Data URI') }}</option>
                </select>
              </div>

              <div>
                <div class="flex justify-between text-xs font-semibold text-[var(--color-text-secondary)] mb-1">
                  <span>{{ t('Compression Quality') }}</span>
                  <span class="text-[var(--color-accent)]">{{ config.compress_quality }}%</span>
                </div>
                <input type="range" min="10" max="100" v-model.number="config.compress_quality" class="w-full accent-[var(--color-accent)] bg-[var(--bg-input)]" />
              </div>

              <div>
                <label class="block text-xs font-semibold text-[var(--color-text-secondary)] mb-1">{{ t('Max Dimension (Pixels)') }}</label>
                <input type="number" v-model.number="config.max_dimension" :placeholder="t('No Limit')" class="w-full bg-[var(--bg-input)] border border-[var(--color-input-border)] rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-accent)] text-[var(--color-text-primary)]" />
              </div>
            </div>

            <!-- Right Card -->
            <div class="bg-[var(--bg-card)] backdrop-blur-2xl border border-[var(--color-border)] rounded-2xl p-6 space-y-4 shadow-[0_8px_32px_rgba(0,0,0,0.37)]">
              <h3 class="text-sm font-semibold uppercase text-[var(--color-text-secondary)] tracking-wider">{{ t('System Integration') }}</h3>

              <div>
                <label class="block text-xs font-semibold text-[var(--color-text-secondary)] mb-1">
                  {{ t('Inject Hotkey') }}
                  <span v-if="recordingHotkey" class="text-[var(--color-accent)] font-bold ml-1 animate-pulse">{{ t('(Recording...)') }}</span>
                  <span v-else class="text-[var(--color-text-secondary)]/80 normal-case font-normal ml-1">{{ t('(click & press keys)') }}</span>
                </label>
                <div class="flex gap-2">
                  <input type="text" readonly :value="config.global_hotkey" @focus="recordingHotkey = true" @blur="recordingHotkey = false" @keydown="recordHotkeyKeydown" :class="['flex-1 bg-[var(--bg-input)] border rounded-xl px-3 py-2 text-sm focus:outline-none text-[var(--color-text-primary)] font-mono cursor-pointer transition-all', recordingHotkey ? 'border-[var(--color-accent)] shadow-[0_0_0_2px_rgba(41,151,255,0.2)]' : 'border-[var(--color-input-border)] focus:border-[var(--color-accent)]']" />
                  <button type="button" @click="config.global_hotkey = 'Alt+V'" class="px-3 py-2 text-xs font-medium bg-[var(--bg-button)] hover:bg-[var(--bg-button-hover)] text-[var(--color-text-secondary)] rounded-xl transition-colors border border-[var(--color-input-border)]">{{ t('Reset') }}</button>
                </div>
              </div>
              <div>
                <label class="block text-xs font-semibold text-[var(--color-text-secondary)] mb-1">
                  {{ t('Screenshot Hotkey') }}
                  <span v-if="recordingShot" class="text-[var(--color-accent)] font-bold ml-1 animate-pulse">{{ t('(Recording...)') }}</span>
                  <span v-else class="text-[var(--color-text-secondary)]/80 normal-case font-normal ml-1">{{ t('(region capture)') }}</span>
                </label>
                <div class="flex gap-2">
                  <input type="text" readonly :value="config.screenshot_hotkey" @focus="recordingShot = true" @blur="recordingShot = false" @keydown="recordShotKeydown" :class="['flex-1 bg-[var(--bg-input)] border rounded-xl px-3 py-2 text-sm focus:outline-none text-[var(--color-text-primary)] font-mono cursor-pointer transition-all', recordingShot ? 'border-[var(--color-accent)] shadow-[0_0_0_2px_rgba(41,151,255,0.2)]' : 'border-[var(--color-input-border)] focus:border-[var(--color-accent)]']" />
                  <button type="button" @click="config.screenshot_hotkey = 'Alt+Shift+S'" class="px-3 py-2 text-xs font-medium bg-[var(--bg-button)] hover:bg-[var(--bg-button-hover)] text-[var(--color-text-secondary)] rounded-xl transition-colors border border-[var(--color-input-border)]">{{ t('Reset') }}</button>
                </div>
              </div>

              <div>
                <label class="block text-xs font-semibold text-[var(--color-text-secondary)] mb-1">{{ t('Injection Mode') }}</label>
                <select v-model="config.injection_mode" class="w-full bg-[var(--bg-input)] border border-[var(--color-input-border)] rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-accent)] text-[var(--color-text-primary)]">
                  <option value="auto">{{ t('Auto — per-app strategy (recommended)') }}</option>
                  <option value="direct">{{ t('Direct — type the path (no clipboard)') }}</option>
                  <option value="copy">{{ t('Copy Only — manual Ctrl+V') }}</option>
                </select>
              </div>

              <div class="flex items-center justify-between py-1">
                <div>
                  <span class="block text-sm font-medium text-[var(--color-text-primary)]">{{ t('Wrap in Single Quotes') }}</span>
                  <span class="block text-xs text-[var(--color-text-secondary)]">{{ t("Wrap generated link in 'quotes'") }}</span>
                </div>
                <label class="relative inline-flex items-center cursor-pointer">
                  <input type="checkbox" v-model="config.wrap_single_quotes" class="sr-only peer" />
                  <div class="w-11 h-6 bg-[var(--bg-toggle)] rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-[var(--color-toggle-knob)] after:border-[var(--color-toggle-knob)] after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-[var(--color-accent)]"></div>
                </label>
              </div>

              <div class="flex items-center justify-between py-1">
                <div>
                  <span class="block text-sm font-medium text-[var(--color-text-primary)]">{{ t('Launch on Boot') }}</span>
                  <span class="block text-xs text-[var(--color-text-secondary)]">{{ t('Start img2cli automatically') }}</span>
                </div>
                <label class="relative inline-flex items-center cursor-pointer">
                  <input type="checkbox" v-model="config.launch_on_boot" class="sr-only peer" />
                  <div class="w-11 h-6 bg-[var(--bg-toggle)] rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-[var(--color-toggle-knob)] after:border-[var(--color-toggle-knob)] after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-[var(--color-accent)]"></div>
                </label>
              </div>

              <div class="flex items-center justify-between py-1">
                <div>
                  <span class="block text-sm font-medium text-[var(--color-text-primary)]">{{ t('Enable Desktop Notifications') }}</span>
                  <span class="block text-xs text-[var(--color-text-secondary)]">{{ t('Show tips on screenshot success') }}</span>
                </div>
                <label class="relative inline-flex items-center cursor-pointer">
                  <input type="checkbox" v-model="config.enable_notifications" class="sr-only peer" />
                  <div class="w-11 h-6 bg-[var(--bg-toggle)] rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-[var(--color-toggle-knob)] after:border-[var(--color-toggle-knob)] after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-[var(--color-accent)]"></div>
                </label>
              </div>
            </div>
          </div>
          <!-- Interface Theme Selector -->
          <div class="bg-[var(--bg-card)] border border-[var(--color-border)] rounded-2xl p-6 space-y-4 shadow-[0_8px_32px_rgba(0,0,0,0.37)]">
            <h3 class="text-sm font-semibold uppercase text-[var(--color-text-secondary)] tracking-wider">{{ t('Interface Theme') }}</h3>
            <div class="flex items-center gap-3">
              <span class="w-5 h-5 rounded-full border border-[var(--color-border)] shrink-0 shadow-inner" :style="{ backgroundColor: currentTheme.colorAccent }" :title="'Accent: ' + currentTheme.colorAccent"></span>
              <select v-model="config.theme" class="flex-1 bg-[var(--bg-input)] border border-[var(--color-input-border)] rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-accent)] text-[var(--color-text-primary)]">
                <option v-for="(tOpts, name) in themes" :key="name" :value="name">{{ themeLabel(name) }}</option>
              </select>
            </div>
            <div class="flex items-center justify-between py-1">
              <span class="block text-sm font-medium text-[var(--color-text-primary)]">{{ t('Display Language') }}</span>
              <select v-model="config.language" class="w-44 bg-[var(--bg-input)] border border-[var(--color-input-border)] rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-accent)] text-[var(--color-text-primary)]">
                <option value="zh-CN">简体中文</option>
                <option value="en">English</option>
              </select>
            </div>
          </div>

          <!-- Save Directory Config -->
          <div class="bg-[var(--bg-card)] border border-[var(--color-border)] rounded-2xl p-6 space-y-4 shadow-[0_8px_32px_rgba(0,0,0,0.37)]">
            <h3 class="text-sm font-semibold uppercase text-[var(--color-text-secondary)] tracking-wider">{{ t('Advanced Paths') }}</h3>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div>
                <label class="block text-xs font-semibold text-[var(--color-text-secondary)] mb-1">{{ t('Local Temporary Directory') }}</label>
                <input type="text" v-model="config.save_dir" :placeholder="t('Default (Temp Dir/img2cli)')" class="w-full bg-[var(--bg-input)] border border-[var(--color-input-border)] rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-accent)] text-[var(--color-text-primary)]" />
              </div>
              <div>
                <label class="block text-xs font-semibold text-[var(--color-text-secondary)] mb-1">{{ t('Clean Expired Image Files (Days)') }}</label>
                <input type="number" v-model.number="config.clean_keep_days" class="w-full bg-[var(--bg-input)] border border-[var(--color-input-border)] rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-accent)] text-[var(--color-text-primary)]" />
              </div>
            </div>
          </div>

          <!-- Capture Options (6-J / 6-L) -->
          <div class="bg-[var(--bg-card)] border border-[var(--color-border)] rounded-2xl p-6 space-y-4 shadow-[0_8px_32px_rgba(0,0,0,0.37)]">
            <h3 class="text-sm font-semibold uppercase text-[var(--color-text-secondary)] tracking-wider">{{ t('Capture Options') }}</h3>
            <div class="flex items-center justify-between py-1">
              <div>
                <span class="block text-sm font-medium text-[var(--color-text-primary)]">{{ t('Auto-detect windows') }}</span>
                <span class="block text-xs text-[var(--color-text-secondary)]">{{ t('Click to snap the window under the cursor') }}</span>
              </div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" v-model="config.capture_auto_detect" class="sr-only peer" />
                <div class="w-11 h-6 bg-[var(--bg-toggle)] rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-[var(--color-toggle-knob)] after:border-[var(--color-toggle-knob)] after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-[var(--color-accent)]"></div>
              </label>
            </div>
            <div class="flex items-center justify-between py-1">
              <div>
                <span class="block text-sm font-medium text-[var(--color-text-primary)]">{{ t('Show capture hints') }}</span>
              </div>
              <label class="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" v-model="config.capture_show_hints" class="sr-only peer" />
                <div class="w-11 h-6 bg-[var(--bg-toggle)] rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-[var(--color-toggle-knob)] after:border-[var(--color-toggle-knob)] after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-[var(--color-accent)]"></div>
              </label>
            </div>
            <div>
              <div class="flex justify-between text-xs font-semibold text-[var(--color-text-secondary)] mb-1">
                <span>{{ t('Selection border width (px)') }}</span>
                <span class="text-[var(--color-accent)]">{{ config.capture_border_width }}</span>
              </div>
              <input type="range" min="1" max="6" v-model.number="config.capture_border_width" class="w-full accent-[var(--color-accent)] bg-[var(--bg-input)]" />
            </div>
            <div>
              <div class="flex justify-between text-xs font-semibold text-[var(--color-text-secondary)] mb-1">
                <span>{{ t('Mask opacity (%)') }}</span>
                <span class="text-[var(--color-accent)]">{{ config.capture_mask_opacity }}%</span>
              </div>
              <input type="range" min="0" max="90" v-model.number="config.capture_mask_opacity" class="w-full accent-[var(--color-accent)] bg-[var(--bg-input)]" />
            </div>
          </div>

          <div class="flex justify-end pt-2">
            <button @click="saveSettings" class="flex items-center gap-2 bg-[var(--color-accent)] hover:bg-[var(--color-accent-hover)] text-white px-6 py-2.5 rounded-full font-semibold shadow-sm shadow-[var(--color-accent)]/15 active:scale-[0.98] transition-all duration-150 text-sm">
              {{ t('Save Settings') }}
            </button>
          </div>
        </div>

        <!-- Hosts & Targets Tab -->
        <div v-if="activeTab === 'hosts'" class="space-y-6">
          <div>
            <h2 class="text-2xl font-bold tracking-tight text-[var(--color-text-primary)]">{{ t('Hosts & Targets') }}</h2>
            <p class="text-sm text-[var(--color-text-secondary)]">{{ t('Configure remote SSH servers and local workspace directory routing.') }}</p>
          </div>

          <!-- Routing Targets (default host + dynamic rules as one card list, 6-N) -->
          <div class="bg-[var(--bg-card)] backdrop-blur-2xl border border-[var(--color-border)] rounded-2xl p-6 space-y-4 shadow-[0_8px_32px_rgba(0,0,0,0.37)]">
            <div class="flex items-center justify-between border-b border-[var(--color-input-border)] pb-3">
              <h3 class="text-sm font-semibold uppercase text-[var(--color-text-secondary)] tracking-wider">{{ t('Routing Targets') }}</h3>
              <div class="flex items-center gap-2">
                <button
                  @click="openSshLoader"
                  :disabled="loadingSsh"
                  class="bg-white/5 hover:bg-white/10 border border-[var(--color-border)] text-[var(--color-text-primary)] font-semibold px-3 py-1.5 rounded-xl text-xs flex items-center gap-1 active:scale-[0.98] transition-all disabled:opacity-50"
                >
                  <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
                  </svg>
                  {{ t('Load SSH Config') }}
                </button>
                <button
                  @click="showAddTargetModal = true"
                  class="bg-[var(--color-accent)] hover:bg-[var(--color-accent-hover)] text-white font-semibold px-3 py-1.5 rounded-xl text-xs flex items-center gap-1 active:scale-[0.98] transition-all"
                >
                  <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
                  </svg>
                  {{ t('Add Target') }}
                </button>
              </div>
            </div>

            <!-- Target cards (Orca-style, Milestone 6-H; default is a flag on one card, 6-Q) -->
            <div class="space-y-3">
              <div v-for="(target, idx) in (config.targets || [])" :key="target.match_pattern || idx"
                   class="rounded-xl border border-[var(--color-border)] bg-[var(--bg-input)]/40 px-4 py-3 flex items-center gap-3 hover:border-[var(--color-accent)]/40 transition-colors">
                <span class="w-2.5 h-2.5 rounded-full shrink-0"
                      :class="{
                        'bg-emerald-400': targetTest[target.match_pattern] === 'ok' && target.enabled,
                        'bg-red-400': targetTest[target.match_pattern] === 'fail',
                        'bg-amber-400 animate-pulse': targetTest[target.match_pattern] === 'testing',
                        'bg-[var(--color-text-secondary)]/40': !target.enabled || !targetTest[target.match_pattern],
                      }"
                      :title="targetTestTitle(target)"></span>
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2 flex-wrap">
                    <span class="text-sm font-semibold text-[var(--color-text-primary)] truncate">{{ target.match_pattern }}</span>
                    <span v-if="target.type === 'ssh'" class="px-1.5 py-0.5 rounded-md text-[10px] font-semibold uppercase bg-[var(--color-accent)]/10 text-[var(--color-accent)] border border-[var(--color-accent)]/25">SSH</span>
                    <span v-else class="px-1.5 py-0.5 rounded-md text-[10px] font-semibold uppercase bg-[var(--color-text-secondary)]/10 text-[var(--color-text-secondary)] border border-[var(--color-text-secondary)]/25">{{ t('Local') }}</span>
                    <span v-if="target.is_default" class="px-1.5 py-0.5 rounded-md text-[10px] font-semibold bg-emerald-500/10 text-emerald-400 border border-emerald-500/25">{{ t('Default') }}</span>
                  </div>
                  <div class="text-xs text-[var(--color-text-secondary)] font-mono truncate mt-0.5">
                    <template v-if="target.type === 'ssh'">{{ target.username }}@{{ target.host }}:{{ target.port || 22 }}</template>
                    <template v-else>{{ target.local_dir }}</template>
                  </div>
                </div>
                <label class="relative inline-flex items-center cursor-pointer shrink-0">
                  <input type="checkbox" v-model="target.enabled" class="sr-only peer" />
                  <div class="w-9 h-5 bg-[var(--bg-toggle)] rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-[var(--color-toggle-knob)] after:border-[var(--color-toggle-knob)] after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-[var(--color-accent)]"></div>
                </label>
                <div class="flex items-center gap-1.5 shrink-0">
                  <button v-if="target.type === 'ssh'" @click="testTargetCard(target)" :disabled="targetTest[target.match_pattern] === 'testing'"
                          class="px-2 py-1 rounded-lg text-[11px] font-semibold bg-[var(--bg-button)] hover:bg-[var(--bg-button-hover)] text-[var(--color-text-primary)] disabled:opacity-50 transition-colors">{{ targetTest[target.match_pattern] === 'testing' ? t('Testing...') : t('Test') }}</button>
                  <button v-if="target.type === 'ssh'" @click="setAsDefault(idx)"
                          class="px-2 py-1 rounded-lg text-[11px] font-semibold bg-[var(--color-accent)]/10 text-[var(--color-accent)] border border-[var(--color-accent)]/25 hover:bg-[var(--color-accent)]/20 transition-colors">{{ t('Set Default') }}</button>
                  <button @click="editTarget(idx)"
                          class="px-2 py-1 rounded-lg text-[11px] font-semibold bg-[var(--color-text-secondary)]/10 text-[var(--color-text-secondary)] border border-[var(--color-text-secondary)]/25 hover:bg-[var(--color-text-secondary)]/20 transition-colors">{{ t('Edit') }}</button>
                  <button @click="deleteTarget(idx)"
                          class="px-2 py-1 rounded-lg text-[11px] font-semibold bg-red-500/10 text-red-400 border border-red-500/25 hover:bg-red-500/20 transition-colors">{{ t('Delete') }}</button>
                </div>
              </div>
              <div v-if="!(config.targets || []).length" class="text-center py-6 text-[var(--color-text-secondary)] text-xs">{{ t('No routing targets configured. Clipboard uploads will fallback to default host.') }}</div>
            </div>
          </div>

          <div class="flex justify-end pt-2">
            <button @click="saveSettings" class="flex items-center gap-2 bg-[var(--color-accent)] hover:bg-[var(--color-accent-hover)] text-white px-6 py-2.5 rounded-full font-semibold shadow-sm shadow-[var(--color-accent)]/15 active:scale-[0.98] transition-all duration-150 text-sm">
              {{ t('Save Settings') }}
            </button>
          </div>
        </div>

        <!-- System Logs Tab -->
        <div v-if="activeTab === 'logs'" class="space-y-6 flex flex-col h-[calc(100vh-8rem)]">
          <div class="flex justify-between items-center shrink-0">
            <div>
              <h2 class="text-2xl font-bold tracking-tight text-[var(--color-text-primary)]">{{ t('System Logs') }}</h2>
              <p class="text-sm text-[var(--color-text-secondary)]">{{ t('Real-time daemon events and screenshot processing logs.') }}</p>
            </div>
            <div class="flex items-center gap-2">
              <button @click="copyAllLogs" class="bg-[var(--bg-button)] hover:bg-[var(--bg-button-hover)] text-[var(--color-text-primary)] font-semibold px-3 py-1.5 rounded-xl text-xs active:scale-[0.98] transition-all">
                {{ t('Copy All') }}
              </button>
              <button @click="exportLogs" class="bg-[var(--bg-button)] hover:bg-[var(--bg-button-hover)] text-[var(--color-text-primary)] font-semibold px-3 py-1.5 rounded-xl text-xs active:scale-[0.98] transition-all">
                {{ t('Export…') }}
              </button>
              <button @click="logs = []" class="bg-[var(--bg-button)] hover:bg-[var(--bg-button-hover)] text-[var(--color-text-primary)] font-semibold px-3 py-1.5 rounded-xl text-xs active:scale-[0.98] transition-all">
                {{ t('Clear Logs') }}
              </button>
            </div>
          </div>

          <div class="flex-1 bg-[var(--bg-input)] border border-[var(--color-input-border)] rounded-2xl p-4 overflow-y-auto font-mono text-xs text-[var(--color-text-secondary)] space-y-1.5 shadow-inner" ref="logContainer">
            <div v-for="(log, idx) in logs" :key="idx" class="whitespace-pre-wrap leading-relaxed">
              <span class="text-[var(--color-text-secondary)]/80 select-none">[{{ idx + 1 }}]</span> {{ log }}
            </div>
            <div v-if="!logs.length" class="text-[var(--color-text-secondary)]/80 text-center py-12">{{ t('No logs loaded. Press global hotkey to trigger daemon activity.') }}</div>
          </div>
        </div>

      </main>
    </div>

    <!-- Add/Edit Target Modal -->
    <div v-if="showAddTargetModal" class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50">
      <div class="bg-[var(--bg-card)] backdrop-blur-2xl border border-[var(--color-border)] rounded-2xl max-w-lg w-full p-6 space-y-4 shadow-2xl">
        <h3 class="text-lg font-bold text-[var(--color-text-primary)]">{{ editingTargetIndex !== null ? t('Edit Router Target') : t('Add Router Target') }}</h3>
        
        <div class="space-y-3">
          <div class="grid grid-cols-2 gap-3">
            <div>
              <label class="block text-xs font-semibold text-[var(--color-text-secondary)] mb-1">{{ t('Target Type') }}</label>
              <select v-model="tempTarget.type" class="w-full bg-[var(--bg-input)] border border-[var(--color-input-border)] rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-accent)] text-[var(--color-text-primary)]">
                <option value="ssh">{{ t('SSH (Remote Server)') }}</option>
                <option value="local">{{ t('Local Folder') }}</option>
              </select>
            </div>
            <div>
              <label class="block text-xs font-semibold text-[var(--color-text-secondary)] mb-1">{{ t('Host Name / Alias') }}</label>
              <input type="text" v-model="tempTarget.match_pattern" :placeholder="t('e.g. GPU-90, WSL')" class="w-full bg-[var(--bg-input)] border border-[var(--color-input-border)] rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-accent)] text-[var(--color-text-primary)]" />
            </div>
          </div>

          <!-- SSH Target Fields -->
          <div v-if="tempTarget.type === 'ssh'" class="grid grid-cols-2 gap-3">
            <div>
              <label class="block text-xs font-semibold text-[var(--color-text-secondary)] mb-1">{{ t('Host IP / Address') }}</label>
              <input type="text" v-model="tempTarget.host" class="w-full bg-[var(--bg-input)] border border-[var(--color-input-border)] rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-accent)] text-[var(--color-text-primary)]" />
            </div>
            <div>
              <label class="block text-xs font-semibold text-[var(--color-text-secondary)] mb-1">{{ t('Port') }}</label>
              <input type="number" v-model.number="tempTarget.port" class="w-full bg-[var(--bg-input)] border border-[var(--color-input-border)] rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-accent)] text-[var(--color-text-primary)]" />
            </div>
            <div>
              <label class="block text-xs font-semibold text-[var(--color-text-secondary)] mb-1">{{ t('Username') }}</label>
              <input type="text" v-model="tempTarget.username" class="w-full bg-[var(--bg-input)] border border-[var(--color-input-border)] rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-accent)] text-[var(--color-text-primary)]" />
            </div>
            <div>
              <label class="block text-xs font-semibold text-[var(--color-text-secondary)] mb-1">{{ t('Remote Copy Destination Folder') }}</label>
              <input type="text" v-model="tempTarget.remote_dir" class="w-full bg-[var(--bg-input)] border border-[var(--color-input-border)] rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-accent)] text-[var(--color-text-primary)]" />
            </div>
            <div>
              <label class="block text-xs font-semibold text-[var(--color-text-secondary)] mb-1">{{ t('Password') }} <span class="text-[var(--color-text-secondary)]/80 normal-case font-normal">{{ t('(OS keyring)') }}</span></label>
              <input type="password" v-model="tempTarget.password" :placeholder="tempTargetHasPassword ? t('●●●●●● (saved) — type a new one to update') : t('blank: uses your SSH key (~/.ssh)')" class="w-full bg-[var(--bg-input)] border border-[var(--color-input-border)] rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-accent)] text-[var(--color-text-primary)]" />
              <div class="flex items-center gap-2 mt-1.5">
                <input type="checkbox" id="target-remember-pwd" v-model="tempTarget.remember_password" class="accent-[var(--color-accent)] rounded bg-[var(--bg-input)] border-[var(--color-input-border)]" />
                <label for="target-remember-pwd" class="text-xs font-medium text-[var(--color-text-secondary)] cursor-pointer select-none">{{ t('Remember Password (OS Keyring)') }}</label>
              </div>
              <div class="text-[11px] mt-1 flex items-center gap-2">
                <template v-if="tempTargetHasPassword">
                  <span class="text-emerald-400">{{ t('✓ Password saved (keyring)') }}</span>
                  <button type="button" @click="clearTargetPassword" class="text-red-400/80 hover:text-red-400 underline">{{ t('clear') }}</button>
                </template>
                <span v-else class="text-[var(--color-text-secondary)]">{{ t('No password → will use your SSH key (~/.ssh)') }}</span>
              </div>
            </div>
          </div>

          <!-- Local Target Fields -->
          <div v-if="tempTarget.type === 'local'">
            <label class="block text-xs font-semibold text-[var(--color-text-secondary)] mb-1">{{ t('Local Copy Destination Folder') }}</label>
            <input type="text" v-model="tempTarget.local_dir" :placeholder="t('e.g. C:\\users\\docs\\images')" class="w-full bg-[var(--bg-input)] border border-[var(--color-input-border)] rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-accent)] text-[var(--color-text-primary)]" />
          </div>
        </div>

        <div class="flex justify-end gap-3 pt-3 border-t border-[var(--color-input-border)]">
          <button @click="closeTargetModal" class="bg-[var(--bg-button)] hover:bg-[var(--bg-button-hover)] text-[var(--color-text-primary)] px-4 py-2 rounded-xl text-xs font-semibold">{{ t('Cancel') }}</button>
          <button @click="saveTarget" class="bg-[var(--color-accent)] hover:bg-[var(--color-accent-hover)] text-white px-4 py-2 rounded-xl text-xs font-semibold">{{ t('Save') }}</button>
        </div>
      </div>
    </div>

    <!-- SSH Config Loader Modal -->
    <div v-if="showSshModal" class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50">
      <div class="bg-[var(--bg-card)] backdrop-blur-2xl border border-[var(--color-border)] rounded-2xl max-w-lg w-full p-6 space-y-4 shadow-2xl">
        <h3 class="text-lg font-bold text-[var(--color-text-primary)]">{{ t('Load OpenSSH config') }}</h3>
        <div class="flex items-center gap-2">
          <input type="text" v-model="sshConfigPath" placeholder="~/.ssh/config" class="flex-1 bg-[var(--bg-input)]/60 border border-[var(--color-border)] rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-accent)] text-[var(--color-text-primary)] font-mono" />
          <button @click="browseSshConfig" :disabled="loadingSsh" class="bg-white/5 hover:bg-white/10 border border-[var(--color-border)] text-[var(--color-text-primary)] font-semibold px-3 py-2 rounded-xl text-xs disabled:opacity-50 whitespace-nowrap">{{ t('Browse…') }}</button>
          <button @click="openSshLoader" :disabled="loadingSsh" class="bg-white/5 hover:bg-white/10 border border-[var(--color-border)] text-[var(--color-text-primary)] font-semibold px-3 py-2 rounded-xl text-xs disabled:opacity-50 whitespace-nowrap">{{ t('Load') }}</button>
        </div>
        <input type="text" v-model="sshSearch" :placeholder="t('Search hosts (alias / host / user)...')" class="w-full bg-[var(--bg-input)]/60 border border-[var(--color-border)] rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-accent)] text-[var(--color-text-primary)]" />
        <div class="max-h-72 overflow-y-auto space-y-1 pr-1">
          <label v-for="{ h, i } in filteredSshHosts" :key="i" class="flex items-center gap-3 px-3 py-2 rounded-lg hover:bg-white/5 cursor-pointer">
            <input type="checkbox" v-model="sshSelected[i]" class="accent-[var(--color-accent)] w-4 h-4" />
            <div class="flex-1 min-w-0">
              <div class="text-sm font-semibold text-[var(--color-text-primary)] truncate">{{ h.alias }}</div>
              <div class="text-xs text-[var(--color-text-secondary)] truncate font-mono">{{ h.username }}@{{ h.host }}:{{ h.port }}</div>
            </div>
          </label>
          <div v-if="!filteredSshHosts.length" class="text-[var(--color-text-secondary)] text-center py-8 text-sm">{{ t('No hosts found.') }}</div>
        </div>
        <div class="flex items-center justify-between pt-3 border-t border-[var(--color-border)]">
          <button @click="toggleAllSsh(true)" class="text-xs font-semibold text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)] transition-colors">{{ t('Select All') }}</button>
          <div class="flex gap-3">
            <button @click="closeSshModal" class="bg-white/5 hover:bg-white/10 text-[var(--color-text-primary)] px-4 py-2 rounded-xl text-xs font-semibold">{{ t('Cancel') }}</button>
            <button @click="importSshSelected" class="bg-[var(--color-accent)] hover:bg-[var(--color-accent-hover)] text-white px-4 py-2 rounded-xl text-xs font-semibold">{{ t('Import Selected') }}</button>
          </div>
        </div>
      </div>
    </div>

    <!-- Notification Toast -->
    <div v-if="toast.show" :class="['fixed bottom-6 right-6 p-4 pr-3 rounded-xl shadow-2xl flex items-start gap-3 border z-50 transition-all duration-300 max-w-sm', toast.isError ? 'bg-red-950/90 border-red-800 text-red-200' : 'bg-emerald-950/90 border-emerald-800 text-emerald-200']">
      <svg v-if="toast.isError" class="w-5 h-5 text-red-400 mt-0.5 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
      </svg>
      <svg v-else class="w-5 h-5 text-emerald-400 mt-0.5 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
      <span class="text-sm font-medium leading-relaxed flex-1">{{ toast.message }}</span>
      <button @click="closeToast" class="shrink-0 opacity-60 hover:opacity-100 transition-opacity" aria-label="Dismiss">
        <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" /></svg>
      </button>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, nextTick, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { open as openDialog, save as saveDialog } from '@tauri-apps/plugin-dialog';
import { ZH, THEME_ZH } from './strings.js';

// UI localization (Milestone 6-I): keys are the English source strings; zh-CN
// swaps them via the ZH dictionary, anything else shows the key as-is.
const APP_VERSION = '0.4.4';
const lang = ref('zh-CN');
const t = (s) => (lang.value === 'zh-CN' && Object.prototype.hasOwnProperty.call(ZH, s) ? ZH[s] : s);

// Active Tab
const activeTab = ref('general');

watch(activeTab, (newTab) => {
  // If the user navigates away or switches tabs, revert the unsaved hotkeys to the last saved values
  if (config.value) {
    if (lastSavedGlobalHotkey.value) {
      config.value.global_hotkey = lastSavedGlobalHotkey.value;
    }
    if (lastSavedScreenshotHotkey.value) {
      config.value.screenshot_hotkey = lastSavedScreenshotHotkey.value;
    }
  }
});

// App Configuration
const config = ref({
  save_dir: '',
  output_format: 'markdown',
  compress_quality: 80,
  max_dimension: 1024,
  workspace_aware: false,
  wrap_single_quotes: false,
  launch_on_boot: true,
  enable_notifications: true,
  global_hotkey: 'Alt+V',
  screenshot_hotkey: 'Alt+Shift+S',
  upload_strategy: 'eager',
  injection_mode: 'auto',
  clean_keep_days: 1,
  theme: 'dracula',
  language: 'zh-CN',
  capture_auto_detect: true,
  capture_remember_region: true,
  capture_show_hints: true,
  capture_border_width: 2,
  capture_mask_opacity: 45,
  last_capture_rect: null,
  ssh: {
    enabled: false,
    host: '',
    port: 22,
    username: '',
    remote_dir: '',
    match_pattern: ''
  },
  targets: []
});

// Theme Specifications mapping
const themes = {
  'apple-dark': {
    bgApp: '#08080c',
    bgSidebar: 'rgba(255, 255, 255, 0.04)',
    bgCard: 'rgba(255, 255, 255, 0.05)',
    colorBorder: 'rgba(255, 255, 255, 0.1)',
    colorAccent: '#2997ff',
    colorAccentHover: '#40a4ff',
    colorAccentDim: 'rgba(41, 151, 255, 0.1)',
    textPrimary: '#f8fafc',
    textSecondary: '#94a3b8',
    bgInput: '#020617',
    colorInputBorder: '#1e293b',
    bgToggle: 'rgba(255,255,255,0.12)', colorToggleKnob: '#cbd5e1', bgButton: '#1e293b', bgButtonHover: '#334155'
  },
  'apple-light': {
    bgApp: '#e9ebef',
    bgSidebar: 'rgba(0, 0, 0, 0.03)',
    bgCard: '#ffffff',
    colorBorder: 'rgba(0, 0, 0, 0.08)',
    colorAccent: '#0071e3',
    colorAccentHover: '#0077ed',
    colorAccentDim: 'rgba(0, 113, 227, 0.08)',
    textPrimary: '#1d1d1f',
    textSecondary: '#55575c',
    bgInput: '#ffffff',
    colorInputBorder: 'rgba(0, 0, 0, 0.15)',
    bgToggle: 'rgba(0,0,0,0.10)', colorToggleKnob: '#ffffff', bgButton: 'rgba(0,0,0,0.05)', bgButtonHover: 'rgba(0,0,0,0.10)'
  },
  'dracula': {
    // v0.3.12: retuned toward a deep-blue (Catppuccin-Mocha-family) background
    // per the herdr reference — the old #282a36 + desaturated grays read gray.
    bgApp: '#1e1e2e',
    bgSidebar: 'rgba(24, 24, 37, 0.75)',
    bgCard: 'rgba(49, 50, 68, 0.45)',
    colorBorder: 'rgba(108, 112, 134, 0.35)',
    colorAccent: '#bd93f9',
    colorAccentHover: '#ff79c6',
    colorAccentDim: 'rgba(189, 147, 249, 0.1)',
    textPrimary: '#f8f8f2',
    textSecondary: '#6272a4',
    bgInput: '#181825',
    colorInputBorder: '#45475a',
    bgToggle: 'rgba(98,114,164,0.30)', colorToggleKnob: '#f8f8f2', bgButton: '#45475a', bgButtonHover: '#585b78'
  },
  'nord': {
    bgApp: '#2e3440',
    bgSidebar: 'rgba(76, 86, 106, 0.3)',
    bgCard: 'rgba(59, 66, 82, 0.4)',
    colorBorder: 'rgba(76, 86, 106, 0.3)',
    colorAccent: '#88c0d0',
    colorAccentHover: '#8fbcbb',
    colorAccentDim: 'rgba(136, 192, 208, 0.1)',
    textPrimary: '#eceff4',
    textSecondary: '#d8dee9',
    bgInput: '#242933',
    colorInputBorder: '#3b4252',
    bgToggle: 'rgba(76,86,106,0.40)', colorToggleKnob: '#eceff4', bgButton: '#3b4252', bgButtonHover: '#434c5e'
  },
  'gruvbox': {
    bgApp: '#282828',
    bgSidebar: 'rgba(50, 48, 47, 0.5)',
    bgCard: 'rgba(60, 56, 54, 0.5)',
    colorBorder: 'rgba(102, 92, 84, 0.4)',
    colorAccent: '#fe8019',
    colorAccentHover: '#fabd2f',
    colorAccentDim: 'rgba(254, 128, 25, 0.1)',
    textPrimary: '#fbf1c7',
    textSecondary: '#a89984',
    bgInput: '#1d2021',
    colorInputBorder: '#3c3836',
    bgToggle: 'rgba(102,92,84,0.40)', colorToggleKnob: '#fbf1c7', bgButton: '#3c3836', bgButtonHover: '#504945'
  },
  'cyberpunk': {
    bgApp: '#0f0f1b',
    bgSidebar: 'rgba(18, 18, 32, 0.6)',
    bgCard: 'rgba(26, 26, 46, 0.5)',
    colorBorder: 'rgba(0, 240, 255, 0.15)',
    colorAccent: '#ff007f',
    colorAccentHover: '#00f0ff',
    colorAccentDim: 'rgba(255, 0, 127, 0.1)',
    textPrimary: '#ffffff',
    textSecondary: '#00f0ff',
    bgInput: '#0a0a14',
    colorInputBorder: 'rgba(0, 240, 255, 0.3)',
    bgToggle: 'rgba(0,240,255,0.20)', colorToggleKnob: '#00f0ff', bgButton: '#1a1a2e', bgButtonHover: '#16213e'
  }
};

const currentTheme = computed(() => {
  const name = config.value?.theme || 'apple-dark';
  return themes[name] || themes['apple-dark'];
});

// Pretty label for the theme <select> options ("apple-dark" -> "Apple Dark").
const themeLabel = (name) =>
  lang.value === 'zh-CN' && Object.prototype.hasOwnProperty.call(THEME_ZH, name)
    ? THEME_ZH[name]
    : name.split('-').map((w) => w[0].toUpperCase() + w.slice(1)).join(' ');

// Language follows config.language (set on load, switched live by the dropdown).
watch(
  () => config.value && config.value.language,
  (v) => { lang.value = v || 'zh-CN'; },
  { immediate: true }
);

// Logs Container & History
const logs = ref([]);
const logContainer = ref(null);

// System Logs toolbar actions (copy all / export to file)
async function copyAllLogs() {
  try {
    await invoke('copy_logs');
    showToast(t('Logs copied to clipboard.'));
  } catch (err) {
    showToast(`${t('Failed to copy logs:')} ${err}`, true);
  }
}

async function exportLogs() {
  try {
    const stamp = new Date().toISOString().slice(0, 10).replaceAll('-', '');
    const path = await saveDialog({
      defaultPath: `img2cli-logs-${stamp}.log`,
      filters: [{ name: 'Log', extensions: ['log', 'txt'] }]
    });
    if (!path) return;
    await invoke('write_logs', { path });
    showToast(`${t('Logs exported to:')} ${path}`);
  } catch (err) {
    showToast(`${t('Failed to export logs:')} ${err}`, true);
  }
}

// Add/Edit Target Modal State
const showAddTargetModal = ref(false);
const editingTargetIndex = ref(null);
const tempTarget = ref({
  enabled: true,
  type: 'ssh',
  match_pattern: '',
  host: '',
  port: 22,
  username: '',
  remote_dir: '',
  local_dir: '',
  password: ''
});

// Target-modal password state (OS keyring; never in config.toml).
const tempTargetHasPassword = ref(false);
const recordingHotkey = ref(false);
const lastSavedGlobalHotkey = ref('');
const lastSavedScreenshotHotkey = ref('');

// ---- OpenSSH config loader ----
const sshHosts = ref([]);
const sshSelected = ref([]); // parallel boolean array (index -> selected)
const showSshModal = ref(false);
const sshSearch = ref('');
const loadingSsh = ref(false);
const sshConfigPath = ref('~/.ssh/config');

const filteredSshHosts = computed(() => {
  const q = sshSearch.value.trim().toLowerCase();
  const all = sshHosts.value.map((h, i) => ({ h, i }));
  if (!q) return all;
  return all.filter(({ h }) =>
    h.alias.toLowerCase().includes(q) ||
    h.host.toLowerCase().includes(q) ||
    h.username.toLowerCase().includes(q)
  );
});

// Toast Manager
const toast = ref({
  show: false,
  message: '',
  isError: false
});

let toastTimer = null;
const showToast = (msg, isErr = false) => {
  toast.value.message = msg;
  toast.value.isError = isErr;
  toast.value.show = true;
  if (toastTimer) { clearTimeout(toastTimer); toastTimer = null; }
  // Error/warning toasts stay visible until dismissed; success auto-hides.
  if (!isErr) {
    toastTimer = setTimeout(() => { toast.value.show = false; }, 4000);
  }
};
const closeToast = () => {
  if (toastTimer) { clearTimeout(toastTimer); toastTimer = null; }
  toast.value.show = false;
};

// ---- OpenSSH config loader actions ----
const openSshLoader = async () => {
  loadingSsh.value = true;
  try {
    const hosts = await invoke('load_ssh_config', { path: sshConfigPath.value });
    sshHosts.value = hosts || [];
    sshSelected.value = sshHosts.value.map(() => false);
    sshSearch.value = '';
    showSshModal.value = true;
  } catch (err) {
    showToast(`${t('Failed to load SSH config:')} ${err}`, true);
  } finally {
    loadingSsh.value = false;
  }
};

const browseSshConfig = async () => {
  try {
    const selected = await openDialog({
      title: 'Select OpenSSH config file',
      multiple: false,
      directory: false,
      filters: [{ name: 'All files', extensions: ['*'] }],
    });
    if (typeof selected === 'string') {
      sshConfigPath.value = selected;
      await openSshLoader();
    }
  } catch (err) {
    showToast(`${t('Failed to open file dialog:')} ${err}`, true);
  }
};

const closeSshModal = () => {
  showSshModal.value = false;
  sshHosts.value = [];
  sshSelected.value = [];
  sshSearch.value = '';
};

const toggleAllSsh = (val) => {
  const next = sshSelected.value.slice();
  filteredSshHosts.value.forEach(({ i }) => { next[i] = val; });
  sshSelected.value = next;
};

const importSshSelected = () => {
  const remoteDir = config.value.ssh?.remote_dir || '/tmp/img2cli';
  let added = 0;
  let skipped = 0;
  sshHosts.value.forEach((h, i) => {
    if (!sshSelected.value[i]) return;
    
    // Deduplicate: check if there's already a target with the same type and match_pattern
    const exists = config.value.targets.some(tgt =>
      tgt.type === 'ssh' &&
      tgt.match_pattern.toLowerCase() === h.alias.toLowerCase()
    );
    
    if (exists) {
      skipped += 1;
      return;
    }
    
    config.value.targets.push({
      enabled: true,
      type: 'ssh',
      match_pattern: h.alias,
      host: h.host,
      port: h.port,
      username: h.username,
      remote_dir: remoteDir,
      local_dir: '',
      remember_password: true
    });
    added += 1;
  });
  closeSshModal();
  if (skipped > 0) {
    showToast(`${t('Imported')} ${added} ${t('host(s),')} ${skipped} ${t('duplicate(s) skipped')}`);
  } else {
    showToast(`${t('Imported')} ${added} ${t('host(s) as router targets')}`);
  }
};

// Mark one target as the default destination (6-Q): a flag on the card, not
// a copy into config.ssh — later edits to the card can't diverge from what
// routing uses.
const setAsDefault = (index) => {
  const tgt = config.value.targets[index];
  if (!tgt || tgt.type !== 'ssh') return;
  config.value.targets.forEach((t) => { t.is_default = false; });
  tgt.is_default = true;
  showToast(`"${tgt.match_pattern}" ${t('set as the default SSH host.')}`);
};

// Load Configurations
const loadConfig = async () => {
  try {
    const data = await invoke('get_config');
    // Ensure all subfields exist to avoid null errors
    if (!data.ssh) {
      data.ssh = { enabled: false, host: '', port: 22, username: '', remote_dir: '', match_pattern: '', remember_password: true };
    } else if (data.ssh.remember_password === undefined) {
      data.ssh.remember_password = true; // default to true if missing
    }
    if (!data.targets) {
      data.targets = [];
    } else {
      data.targets.forEach(tgt => {
        if (tgt.remember_password === undefined) {
          tgt.remember_password = true; // default to true if missing
        }
      });
    }
    config.value = data;
    lastSavedGlobalHotkey.value = data.global_hotkey || '';
    lastSavedScreenshotHotkey.value = data.screenshot_hotkey || '';
  } catch (err) {
    showToast(`${t('Failed to load configuration:')} ${err}`, true);
  }
};

// Save Configurations
const saveSettings = async () => {
  try {
    await invoke('save_config', { config: config.value });
    lastSavedGlobalHotkey.value = config.value.global_hotkey;
    lastSavedScreenshotHotkey.value = config.value.screenshot_hotkey;
    showToast(t('Settings saved successfully!'));
  } catch (err) {
    showToast(`${t('Failed to save settings:')} ${err}`, true);
  }
};

// Global hotkey recorder: click the field, then press a key combo.
const recordHotkeyKeydown = (e) => {
  if (!recordingHotkey.value) return;
  e.preventDefault();
  if (e.key === 'Escape') { e.target.blur(); return; }
  
  const mods = [];
  if (e.ctrlKey) mods.push('Control');
  if (e.altKey) mods.push('Alt');
  if (e.shiftKey) mods.push('Shift');
  if (e.metaKey) mods.push('Super');

  if (['Control', 'Alt', 'Shift', 'Meta'].includes(e.key)) {
    config.value.global_hotkey = mods.join('+');
    return;
  }

  let key = e.key;
  if (key === ' ') {
    key = 'Space';
  } else if (key.length === 1) {
    key = key.toUpperCase();
  } else {
    key = key.charAt(0).toUpperCase() + key.slice(1);
  }

  mods.push(key);
  config.value.global_hotkey = mods.join('+');
  e.target.blur();
};

// ---- Screenshot (region-capture) hotkey recorder ----
const recordingShot = ref(false);
const recordShotKeydown = (e) => {
  if (!recordingShot.value) return;
  e.preventDefault();
  if (e.key === 'Escape') { e.target.blur(); return; }

  const mods = [];
  if (e.ctrlKey) mods.push('Control');
  if (e.altKey) mods.push('Alt');
  if (e.shiftKey) mods.push('Shift');
  if (e.metaKey) mods.push('Super');

  if (['Control', 'Alt', 'Shift', 'Meta'].includes(e.key)) {
    config.value.screenshot_hotkey = mods.join('+');
    return;
  }

  let key = e.key;
  if (key === ' ') {
    key = 'Space';
  } else if (key.length === 1) {
    key = key.toUpperCase();
  } else {
    key = key.charAt(0).toUpperCase() + key.slice(1);
  }

  mods.push(key);
  config.value.screenshot_hotkey = mods.join('+');
  e.target.blur();
};

// ---- Region-capture overlay (the ?capture=1 window) ----
// Snipaste-style: drag to draw a selection; it then persists with 8 resize
// handles + move, editable until the user confirms (✓ button / Enter) or
// cancels (✕ / Esc). Clicking outside the selection starts a new one.
const captureMode = ref(false);
const capturedImageSrc = ref('');
const rect = ref({ x: 0, y: 0, w: 0, h: 0 });               // normalized selection, CSS px
const hasRect = computed(() => rect.value.w >= 4 && rect.value.h >= 4);
const capAction = ref(null);                                  // 'draw' | 'move' | 'resize' | null
const capHandle = ref(null);
const capOrigin = ref({ mx: 0, my: 0, rect: null });

const rectStyle = computed(() => ({ left: rect.value.x + 'px', top: rect.value.y + 'px', width: rect.value.w + 'px', height: rect.value.h + 'px' }));
const toolbarStyle = computed(() => {
  const below = rect.value.y + rect.value.h + 40 <= window.innerHeight;
  return { right: '0px', top: below ? rect.value.h + 8 + 'px' : '-32px' };
});
const handles = ['nw', 'n', 'ne', 'e', 'se', 's', 'sw', 'w'];
const handlePos = { nw: [0, 0], n: [0.5, 0], ne: [1, 0], e: [1, 0.5], se: [1, 1], s: [0.5, 1], sw: [0, 1], w: [0, 0.5] };
const handleStyle = (hd) => {
  const [fx, fy] = handlePos[hd];
  return { left: `calc(${fx * 100}% - 5px)`, top: `calc(${fy * 100}% - 5px)` };
};
const handleCursorClass = (hd) => ({
  nw: 'cursor-nwse-resize', se: 'cursor-nwse-resize',
  ne: 'cursor-nesw-resize', sw: 'cursor-nesw-resize',
  n: 'cursor-ns-resize', s: 'cursor-ns-resize',
  e: 'cursor-ew-resize', w: 'cursor-ew-resize',
}[hd]);
const clamp = (v, lo, hi) => Math.max(lo, Math.min(hi, v));

// Auto window detection (6-J/6-S): rects from get_window_rects in Z order
// with child ELEMENTS right after their parent (Windows) — all candidates
// containing the cursor form a drill-down list; Tab cycles it.
const winRects = ref([]);
const hoverRect = ref(null);
const downHover = ref(null); // window under the cursor at mousedown (6-P)
const cursorCands = ref([]); // candidates containing the cursor, list order
const candIdx = ref(0);
const candidatesAt = (mx, my) =>
  winRects.value.filter(
    (r) => mx >= r.x && mx <= r.x + r.w && my >= r.y && my <= r.y + r.h
  );
const cycleCandidate = (dir) => {
  if (!cursorCands.value.length) return;
  candIdx.value = (candIdx.value + dir + cursorCands.value.length) % cursorCands.value.length;
  // Snipaste semantics: Tab SWITCHES the selection to the cycled candidate —
  // it must work with or without an existing selection (a selection existing
  // must never empty the candidate list).
  const r = cursorCands.value[candIdx.value];
  hoverRect.value = null;
  // The adopted selection is UNCONFIRMED (T1): inside-drag redraws until ✓.
  if (r) { rect.value = { x: r.x, y: r.y, w: r.w, h: r.h }; confirmed.value = false; }
};
const hoverStyle = computed(() => {
  const r = hoverRect.value;
  if (!r) return {};
  return {
    left: r.x + 'px', top: r.y + 'px', width: r.w + 'px', height: r.h + 'px',
    border: ((config.value.capture_border_width || 2) + 1) + 'px solid #2997ff',
  };
});
// Region history (v0.4.1): Shift+R recalls the newest entry; `,` / `.` cycle.
const captureHistory = ref([]);
const historyCursor = ref(-1);
const applyHistoryRect = () => {
  const r = captureHistory.value[historyCursor.value];
  if (!r) return;
  const px = Math.max(0, r.x);
  const py = Math.max(0, r.y);
  rect.value = {
    x: px, y: py,
    w: Math.max(4, Math.min(r.w, window.innerWidth - px)),
    h: Math.max(4, Math.min(r.h, window.innerHeight - py)),
  };
  hoverRect.value = null;
  confirmed.value = false; // history recall is also an unconfirmed selection (T1)
};
const recallHistory = (idx) => {
  if (!captureHistory.value.length) return;
  historyCursor.value = idx;
  applyHistoryRect();
};
const cycleHistory = (dir) => {
  const n = captureHistory.value.length;
  if (!n) return;
  historyCursor.value = historyCursor.value < 0 ? 0 : (historyCursor.value + dir + n) % n;
  applyHistoryRect();
};

// ── Annotation editor (v0.4.2) — flameshot's value-object model ───────────
// Annotations are plain data objects in OVERLAY coordinates (glued to the
// frozen frame, not the selection, so moving the selection never smears them).
// drawObjects() is the single renderer used by BOTH the live canvas (clipped
// to the selection) and the confirm-time composite at full physical res.
const activeTool = ref('select'); // select | arrow | pen | marker | mosaic | text | rect | ellipse | eraser
// T1 (v0.4.4): a selection existing is NOT confirmation. Only ✓ confirms —
// before that the cursor stays a crosshair and dragging anywhere (incl.
// inside the selection) redraws; after ✓ it becomes the move-arrow inside.
const confirmed = ref(false);
const annots = ref([]);
const undoStack = ref([]);
const redoStack = ref([]);
const toolColor = ref('#ff4d4f');
const toolSize = ref(3); // 1..8 — shared by line width / mosaic blocks / text size
const activeAnnot = ref(null); // in-progress object while dragging
const editingText = ref(null); // { x, y, value }
const textEditor = ref(null);
const colorMenu = ref(false);
const annotCanvas = ref(null);
const frozenImg = new Image();
const palette = ['#ff4d4f', '#faad14', '#52c41a', '#1890ff', '#722ed1', '#ffffff', '#000000'];
const textFontSize = computed(() => 12 + toolSize.value * 3);
const toolButtons = computed(() => [
  { id: 'select', icon: '⬉', tip: t('Select / move') },
  { id: 'arrow', icon: '➶', tip: t('Arrow') },
  { id: 'pen', icon: '✏️', tip: t('Pen') },
  { id: 'marker', icon: '🖍️', tip: t('Marker (highlight)') },
  { id: 'mosaic', icon: '▦', tip: t('Mosaic') },
  { id: 'text', icon: 'T', tip: t('Text') },
  { id: 'rect', icon: '▭', tip: t('Rectangle') },
  { id: 'ellipse', icon: '◯', tip: t('Ellipse') },
  { id: 'eraser', icon: '⌫', tip: t('Eraser (click an object)') },
]);

const pushUndo = () => {
  undoStack.value.push(annots.value.slice());
  if (undoStack.value.length > 50) undoStack.value.shift();
  redoStack.value = [];
};
const undoAnnot = () => {
  if (!undoStack.value.length) return;
  redoStack.value.push(annots.value.slice());
  annots.value = undoStack.value.pop();
  redrawAnnots();
};
const redoAnnot = () => {
  if (!redoStack.value.length) return;
  undoStack.value.push(annots.value.slice());
  annots.value = redoStack.value.pop();
  redrawAnnots();
};
const setTool = (id) => {
  if (editingText.value) commitText();
  activeTool.value = activeTool.value === id && id !== 'select' ? 'select' : id;
  colorMenu.value = false;
};

// Single renderer for every object type (flameshot's process() contract).
const drawObjects = (ctx, list) => {
  for (const a of list) {
    ctx.strokeStyle = a.color;
    ctx.fillStyle = a.color;
    ctx.lineWidth = a.size;
    ctx.lineCap = 'round';
    ctx.lineJoin = 'round';
    if (a.type === 'arrow') {
      const dx = a.x2 - a.x1, dy = a.y2 - a.y1;
      const len = Math.hypot(dx, dy) || 1;
      const ux = dx / len, uy = dy / len;
      const headW = 10 + a.size * 2, headL = Math.min(18 + a.size * 4, len * 0.6);
      const bx = a.x2 - ux * headL, by = a.y2 - uy * headL; // head base
      ctx.beginPath();
      ctx.moveTo(a.x1, a.y1);
      ctx.lineTo(bx, by);
      ctx.stroke();
      ctx.beginPath();
      ctx.moveTo(a.x2, a.y2);
      ctx.lineTo(bx - uy * headW / 2, by + ux * headW / 2);
      ctx.lineTo(bx + uy * headW / 2, by - ux * headW / 2);
      ctx.closePath();
      ctx.fill();
    } else if (a.type === 'pen' || a.type === 'marker') {
      ctx.save();
      if (a.type === 'marker') {
        ctx.globalCompositeOperation = 'multiply';
        ctx.globalAlpha = 0.85;
        ctx.lineWidth = a.size * 4;
      }
      ctx.beginPath();
      a.pts.forEach((p, i) => (i ? ctx.lineTo(p.x, p.y) : ctx.moveTo(p.x, p.y)));
      ctx.stroke();
      ctx.restore();
    } else if (a.type === 'rect') {
      ctx.beginPath();
      ctx.rect(a.x, a.y, a.w, a.h);
      ctx.stroke();
    } else if (a.type === 'ellipse') {
      ctx.beginPath();
      ctx.ellipse(a.x + a.w / 2, a.y + a.h / 2, Math.abs(a.w / 2), Math.abs(a.h / 2), 0, 0, Math.PI * 2);
      ctx.stroke();
    } else if (a.type === 'text') {
      ctx.font = `600 ${a.fontSize}px system-ui, sans-serif`;
      ctx.textBaseline = 'top';
      a.text.split('\n').forEach((line, i) => ctx.fillText(line, a.x, a.y + i * (a.fontSize + 2)));
    } else if (a.type === 'mosaic' && a.w >= 4 && a.h >= 4 && frozenImg.complete && frozenImg.naturalWidth > 0) {
      // Two-step pixelate: downscale to a tiny offscreen, upscale with
      // smoothing off. Source coords are physical (natural size = monitor px).
      const dpr = window.devicePixelRatio || 1;
      const block = Math.max(4, a.size * 3); // css px per block
      const cols = Math.max(1, Math.round(a.w / block));
      const rows = Math.max(1, Math.round(a.h / block));
      const off = document.createElement('canvas');
      off.width = cols; off.height = rows;
      const octx = off.getContext('2d');
      octx.drawImage(frozenImg, a.x * dpr, a.y * dpr, a.w * dpr, a.h * dpr, 0, 0, cols, rows);
      ctx.imageSmoothingEnabled = false;
      ctx.drawImage(off, 0, 0, cols, rows, a.x, a.y, a.w, a.h);
      ctx.imageSmoothingEnabled = true;
    }
  }
};

// Live canvas: full-overlay size at devicePixelRatio, clipped to the selection.
const redrawAnnots = () => {
  const c = annotCanvas.value;
  if (!c) return;
  const dpr = window.devicePixelRatio || 1;
  const W = window.innerWidth, H = window.innerHeight;
  if (c.width !== Math.round(W * dpr) || c.height !== Math.round(H * dpr)) {
    c.width = Math.round(W * dpr);
    c.height = Math.round(H * dpr);
  }
  const ctx = c.getContext('2d');
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clearRect(0, 0, W, H);
  const all = activeAnnot.value ? [...annots.value, activeAnnot.value] : annots.value;
  if (!all.length || !hasRect.value) return;
  ctx.save();
  ctx.beginPath();
  ctx.rect(rect.value.x, rect.value.y, rect.value.w, rect.value.h);
  ctx.clip();
  drawObjects(ctx, all);
  ctx.restore();
};

// Geometric hit-test for the object eraser (topmost first).
const distToSeg = (px, py, x1, y1, x2, y2) => {
  const dx = x2 - x1, dy = y2 - y1;
  const l2 = dx * dx + dy * dy || 1;
  const t = Math.max(0, Math.min(1, ((px - x1) * dx + (py - y1) * dy) / l2));
  return Math.hypot(px - (x1 + t * dx), py - (y1 + t * dy));
};
// Eraser (T3): pen/marker strokes SPLIT at the erased span (Snipaste-style
// partial erase — only the swept segment disappears); shape/text/mosaic
// objects are removed whole. One undo snapshot per sweep.
const ERASE_RADIUS = 10;
const eraseAt = (px, py) => {
  let mutated = false;
  for (let i = annots.value.length - 1; i >= 0; i--) {
    const a = annots.value[i];
    if (a.type === 'pen' || a.type === 'marker') {
      // Keep only the points outside the erase radius; contiguous kept runs
      // become the resulting stroke(s).
      const runs = [];
      let cur = [];
      for (const p of a.pts) {
        if (Math.hypot(p.x - px, p.y - py) <= ERASE_RADIUS) {
          if (cur.length >= 2) runs.push(cur);
          cur = [];
        } else {
          cur.push(p);
        }
      }
      if (cur.length >= 2) runs.push(cur);
      if (runs.length !== 1 || runs[0].length !== a.pts.length) {
        if (!mutated) pushUndo();
        mutated = true;
        annots.value.splice(i, 1);
        // shortest runs first so indices stay stable while inserting
        runs.sort((r1, r2) => r2.length - r1.length);
        runs.forEach((r) => annots.value.splice(i, 0, { ...a, pts: r }));
      }
    } else if (hitAnnotAt(px, py, i)) {
      if (!mutated) pushUndo();
      annots.value.splice(i, 1);
      mutated = true;
    }
  }
  if (mutated) redrawAnnots();
};
const hitAnnotAt = (px, py, idx) => {
  const a = annots.value[idx];
  const pad = Math.max(8, a.size * 2.5);
  if (a.type === 'pen' || a.type === 'marker') {
    for (let s = 1; s < a.pts.length; s++) {
      if (distToSeg(px, py, a.pts[s - 1].x, a.pts[s - 1].y, a.pts[s].x, a.pts[s].y) <= pad) return true;
    }
    return false;
  }
  if (a.type === 'arrow') return distToSeg(px, py, a.x1, a.y1, a.x2, a.y2) <= pad;
  if (a.type === 'rect' || a.type === 'ellipse' || a.type === 'mosaic') {
    return px >= a.x - pad && px <= a.x + a.w + pad && py >= a.y - pad && py <= a.y + a.h + pad;
  }
  if (a.type === 'text') {
    return px >= a.x - 6 && px <= a.x + (a.tw || 120) + 6 && py >= a.y - 6 && py <= a.y + a.text.split('\n').length * (a.fontSize + 2) + 6;
  }
  return false;
};
const hitAnnot = (px, py) => {
  for (let i = annots.value.length - 1; i >= 0; i--) {
    const a = annots.value[i];
    const pad = Math.max(5, a.size * 1.6);
    if (a.type === 'pen' || a.type === 'marker') {
      for (let s = 1; s < a.pts.length; s++) {
        if (distToSeg(px, py, a.pts[s - 1].x, a.pts[s - 1].y, a.pts[s].x, a.pts[s].y) <= pad) return i;
      }
    } else if (a.type === 'arrow') {
      if (distToSeg(px, py, a.x1, a.y1, a.x2, a.y2) <= pad) return i;
    } else if (a.type === 'rect' || a.type === 'ellipse' || a.type === 'mosaic') {
      const x0 = Math.min(a.x, a.x + a.w), x1 = Math.max(a.x, a.x + a.w);
      const y0 = Math.min(a.y, a.y + a.h), y1 = Math.max(a.y, a.y + a.h);
      if (px >= x0 - pad && px <= x1 + pad && py >= y0 - pad && py <= y1 + pad) return i;
    } else if (a.type === 'text') {
      if (px >= a.x - 4 && px <= a.x + (a.tw || 80) && py >= a.y - 4 && py <= a.y + (a.text.split('\n').length * (a.fontSize + 2) + 4)) return i;
    }
  }
  return -1;
};

// A draw tool pressed inside the selection starts an annotation.
const annotMouseDown = (e) => {
  const x = e.clientX, y = e.clientY;
  if (activeTool.value === 'eraser') {
    eraseAt(x, y);
    capAction.value = 'erase'; // hold + sweep to erase continuously
    return;
  }
  if (activeTool.value === 'text') {
    if (editingText.value) commitText();
    editingText.value = { x, y, value: '' };
    // T2 fix: focus via the template ref with retries — querySelector + a
    // single nextTick raced the patch; preventDefault on mousedown keeps the
    // focus from jumping back to the overlay.
    nextTick(() => {
      for (let i = 0; i < 5; i++) {
        setTimeout(() => textEditor.value?.focus(), i * 30);
      }
    });
    return;
  }
  capAction.value = 'annotate';
  const common = { color: toolColor.value, size: toolSize.value };
  if (activeTool.value === 'pen' || activeTool.value === 'marker') {
    activeAnnot.value = { type: activeTool.value, pts: [{ x, y }], ...common };
  } else if (activeTool.value === 'arrow') {
    activeAnnot.value = { type: 'arrow', x1: x, y1: y, x2: x, y2: y, ...common };
  } else {
    // sx/sy = the anchor corner; annotMouseMove normalizes x/y/w/h from it.
    activeAnnot.value = { type: activeTool.value, sx: x, sy: y, x, y, w: 0, h: 0, ...common };
  }
};
const annotMouseMove = (e) => {
  const a = activeAnnot.value;
  if (!a) return;
  if (a.type === 'pen' || a.type === 'marker') a.pts.push({ x: e.clientX, y: e.clientY });
  else if (a.type === 'arrow') { a.x2 = e.clientX; a.y2 = e.clientY; }
  else {
    // Rect-like tools (rect/ellipse/mosaic): normalize live — negative
    // w/h fed into drawImage's source rect breaks the mosaic (and flips
    // shapes); drag from any corner must behave identically.
    const x0 = Math.min(a.sx, e.clientX), x1 = Math.max(a.sx, e.clientX);
    const y0 = Math.min(a.sy, e.clientY), y1 = Math.max(a.sy, e.clientY);
    a.x = x0; a.y = y0; a.w = x1 - x0; a.h = y1 - y0;
  }
  redrawAnnots();
};
const annotMouseUp = () => {
  const a = activeAnnot.value;
  activeAnnot.value = null;
  if (!a) return;
  const valid =
    (a.type === 'pen' || a.type === 'marker') ? a.pts.length > 1 :
    a.type === 'arrow' ? Math.hypot(a.x2 - a.x1, a.y2 - a.y1) > 4 :
    a.type === 'rect' || a.type === 'ellipse' || a.type === 'mosaic' ? Math.abs(a.w) > 4 && Math.abs(a.h) > 4 : false;
  if (valid) {
    if (a.type === 'rect' || a.type === 'ellipse' || a.type === 'mosaic') {
      if (a.w < 0) { a.x += a.w; a.w = -a.w; }
      if (a.h < 0) { a.y += a.h; a.h = -a.h; }
    }
    pushUndo();
    annots.value.push(a);
  }
  redrawAnnots();
};

// Text tool: Enter commits (unless Shift for newline), Esc discards.
const textKeydown = (e) => {
  e.stopPropagation();
  if (e.key === 'Escape') { editingText.value = null; return; }
  if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); commitText(); }
};
const commitText = () => {
  const ed = editingText.value;
  editingText.value = null;
  if (!ed || !ed.value.trim()) return;
  const a = { type: 'text', x: ed.x, y: ed.y, text: ed.value, color: toolColor.value, fontSize: textFontSize.value };
  // measure bbox for the eraser hit-test
  const c = annotCanvas.value;
  if (c) {
    const ctx = c.getContext('2d');
    ctx.font = `600 ${a.fontSize}px system-ui, sans-serif`;
    a.tw = Math.max(...a.text.split('\n').map((l) => ctx.measureText(l).width));
  }
  pushUndo();
  annots.value.push(a);
  redrawAnnots();
};

// Composite the selection (frozen frame + annotations) at PHYSICAL res.
const compositeRegion = () => {
  const dpr = window.devicePixelRatio || 1;
  const c = document.createElement('canvas');
  c.width = Math.max(1, Math.round(rect.value.w * dpr));
  c.height = Math.max(1, Math.round(rect.value.h * dpr));
  const ctx = c.getContext('2d');
  ctx.drawImage(frozenImg,
    rect.value.x * dpr, rect.value.y * dpr, c.width, c.height,
    0, 0, c.width, c.height);
  ctx.scale(dpr, dpr);
  ctx.translate(-rect.value.x, -rect.value.y);
  drawObjects(ctx, annots.value);
  return c.toDataURL('image/png');
};

// Toolbar actions — all operate on the composited region.
const closeOverlay = async () => { try { await invoke('cancel_capture'); } catch (_) {} };
const actionCopy = async () => {
  try {
    await invoke('copy_image', { dataUrl: annots.value.length ? compositeRegion() : plainRegionDataUrl() });
  } catch (_) {}
  closeOverlay();
};
const actionSave = async () => {
  try {
    // T5 (v0.4.4): one-click save — Snipaste-style auto-named file
    // (img2cli_YYYY-MM-DD_HH-mm-ss) straight into the default dir, no dialog.
    const now = new Date();
    const p2 = (n) => String(n).padStart(2, '0');
    const stamp = `${now.getFullYear()}-${p2(now.getMonth() + 1)}-${p2(now.getDate())}_${p2(now.getHours())}-${p2(now.getMinutes())}-${p2(now.getSeconds())}`;
    const dir = config.value.save_dir && config.value.save_dir.trim()
      ? config.value.save_dir.trim().replace(/[\\/]+$/, '')
      : null;
    // Bare filename → Rust resolves it into the default temp img2cli dir.
    const path = dir ? `${dir}/img2cli_${stamp}.png` : `img2cli_${stamp}.png`;
    await invoke('write_image', { path, dataUrl: annots.value.length ? compositeRegion() : plainRegionDataUrl() });
  } catch (err) { console.error('save failed:', err); }
  closeOverlay();
};
const actionPin = async () => {
  try {
    // v0.4.3 fix: single Rust command (store + window build) with daemon
    // logging, so failures surface in the System Logs panel.
    const dataUrl = annots.value.length ? compositeRegion() : plainRegionDataUrl();
    await invoke('pin_image', { dataUrl, w: Math.max(80, rect.value.w), h: Math.max(80, rect.value.h) });
  } catch (err) { console.error('pin failed:', err); }
  closeOverlay();
};
// No annotations → the plain crop equals the frozen-frame region; composite it
// anyway so copy/save/pin share one code path at full physical res.
const plainRegionDataUrl = () => compositeRegion();

// Load the frozen frame + per-session state; reveal the overlay only after
// the frame renders (flash-free). Called at mount AND on every capture-refresh
// (T6 persistent window).
const loadCaptureImage = async () => {
  try {
    const src = await invoke('get_captured_image');
    capturedImageSrc.value = src;
    // The annotation engine needs the frozen frame as a drawable Image
    // (mosaic sampling + compositing). Fresh session state each time.
    frozenImg.src = src;
    annots.value = [];
    undoStack.value = [];
    redoStack.value = [];
    activeTool.value = 'select';
    confirmed.value = false;
    rect.value = { x: 0, y: 0, w: 0, h: 0 };
    hoverRect.value = null;
    cursorCands.value = [];
    redrawAnnots();
    // Capture options + window detection, fetched before the reveal.
    try {
      const cfg = await invoke('get_config');
      ['capture_auto_detect', 'capture_show_hints',
        'capture_border_width', 'capture_mask_opacity'].forEach((k) => {
        if (cfg[k] !== undefined) config.value[k] = cfg[k];
      });
      captureHistory.value = cfg.capture_history || [];
      if (cfg.capture_auto_detect) winRects.value = await invoke('get_window_rects');
    } catch (_) {}
    await nextTick();
    try { await invoke('show_capture_overlay'); } catch (_) {}
  } catch (e) {
    console.error('Failed to load captured image:', e);
  }
};

// Pin window page (?pin=ID).
const pinMode = ref(false);
const pinImg = ref('');
const pinIdNum = ref(0);
const pinBase = ref({ w: 0, h: 0 });
const pinScale = ref(1);
const pinMenu = ref(false);
const closePin = () => { try { invoke('close_pin', { id: pinIdNum.value }); } catch (_) {} };
const startPinDrag = () => { try { invoke('drag_pin', { id: pinIdNum.value }); } catch (_) {} };
const pinCopy = async () => {
  pinMenu.value = false;
  try { await invoke('copy_image', { dataUrl: pinImg.value }); } catch (e) { console.error(e); }
};
const pinSaveAs = async () => {
  pinMenu.value = false;
  try {
    const now = new Date();
    const p2 = (n) => String(n).padStart(2, '0');
    const name = `img2cli_${now.getFullYear()}-${p2(now.getMonth() + 1)}-${p2(now.getDate())}_${p2(now.getHours())}-${p2(now.getMinutes())}-${p2(now.getSeconds())}.png`;
    const path = await invoke('save_image_dialog', { defaultName: name });
    if (path) await invoke('write_image', { path, dataUrl: pinImg.value });
  } catch (e) { console.error(e); }
};
const pinZoom = (e) => {
  pinScale.value = Math.min(5, Math.max(0.2, pinScale.value * (e.deltaY < 0 ? 1.1 : 0.9)));
  try {
    invoke('resize_pin', { id: pinIdNum.value, w: Math.max(80, pinBase.value.w * pinScale.value), h: Math.max(80, pinBase.value.h * pinScale.value) });
  } catch (_) {}
};

// Snipaste-style key guide lines for the bottom-left overlay panel.
const hintLines = computed(() => [
  t('Drag to select · Click a window to snap'),
  t('Tab / Shift+Tab cycle elements'),
  t('Shift+R last region · `,` `.` history'),
  t('WASD move cursor 1px'),
  t('Enter save · Esc cancel'),
]);

// Selection appearance knobs (6-L).
const selBorderStyle = computed(() => ({
  borderWidth: (config.value.capture_border_width || 2) + 'px',
  borderStyle: 'solid',
  borderColor: '#2997ff',
  boxShadow: `0 0 0 9999px rgba(0,0,0,${(config.value.capture_mask_opacity ?? 45) / 100})`,
}));

const capMouseDown = (e) => {
  // 6-P: a press ALWAYS starts a draw — snap arbitration happens on mouseup,
  // so a drag begun inside a detected window still draws a custom selection.
  downHover.value = hoverRect.value;
  startDraw(e);
};
// Drawing is always available — anywhere, including INSIDE an existing
// selection (user request 2026-08-16): a plain mousedown restarts the
// selection; Alt+drag inside keeps the move affordance for adjusting.
const startDraw = (e) => {
  capAction.value = 'draw';
  confirmed.value = false; // a fresh draw returns to the unconfirmed state
  rect.value = { x: e.clientX, y: e.clientY, w: 0, h: 0 };
  capOrigin.value = { mx: e.clientX, my: e.clientY, rect: null };
};
// Selection interaction (T1): a draw tool always annotates; the select tool
// REDRAWS while unconfirmed (free re-selection, Snipaste parity) and MOVES
// only after ✓ confirmed. Handles resize in every state.
const rectMouseDown = (e) => {
  if (activeTool.value !== 'select') { annotMouseDown(e); return; }
  if (!confirmed.value) { startDraw(e); return; }
  startMove(e);
};
// ✓ = confirm ONLY (no upload, no close) — flips the selection into the
// editable state (arrow cursor, move/resize/annotate).
const confirmSelection = () => {
  if (hasRect.value) confirmed.value = true;
};
const startMove = (e) => {
  capAction.value = 'move';
  capOrigin.value = { mx: e.clientX, my: e.clientY, rect: { ...rect.value } };
};
const startResize = (hd, e) => {
  capAction.value = 'resize';
  capHandle.value = hd;
  capOrigin.value = { mx: e.clientX, my: e.clientY, rect: { ...rect.value } };
};
const capMouseMove = (e) => {
  if (!capAction.value) {
    // Candidates are maintained regardless of an existing selection (Tab
    // switches the selection); the passive hover OUTLINE only shows when
    // nothing is selected yet.
    const cands = winRects.value.length ? candidatesAt(e.clientX, e.clientY) : [];
    const changed =
      cands.length !== cursorCands.value.length ||
      cands.some((r, i) => r !== cursorCands.value[i]);
    if (changed) {
      cursorCands.value = cands;
      candIdx.value = 0;
    }
    hoverRect.value = (!hasRect.value && cursorCands.value[candIdx.value]) || null;
    return;
  }
  if (capAction.value === 'annotate') { annotMouseMove(e); return; }
  if (capAction.value === 'erase') { eraseAt(e.clientX, e.clientY); return; }
  const mx = e.clientX, my = e.clientY;
  const winW = window.innerWidth, winH = window.innerHeight;
  if (capAction.value === 'draw') {
    const o = capOrigin.value;
    rect.value = { x: Math.min(o.mx, mx), y: Math.min(o.my, my), w: Math.abs(mx - o.mx), h: Math.abs(my - o.my) };
  } else if (capAction.value === 'move') {
    const o = capOrigin.value;
    rect.value = {
      x: clamp(o.rect.x + (mx - o.mx), 0, winW - o.rect.w),
      y: clamp(o.rect.y + (my - o.my), 0, winH - o.rect.h),
      w: o.rect.w, h: o.rect.h,
    };
  } else if (capAction.value === 'resize') {
    const o = capOrigin.value;
    const hd = capHandle.value;
    let { x, y, w, h } = o.rect;
    const right = x + w, bottom = y + h;
    if (hd.includes('w')) { x = clamp(mx, 0, right - 4); w = right - x; }
    if (hd.includes('e')) { w = Math.max(4, clamp(mx, 0, winW) - x); }
    if (hd.includes('n')) { y = clamp(my, 0, bottom - 4); h = bottom - y; }
    if (hd.includes('s')) { h = Math.max(4, clamp(my, 0, winH) - y); }
    rect.value = { x, y, w, h };
  }
};
const capMouseUp = () => {
  if (capAction.value === 'annotate') { annotMouseUp(); capAction.value = null; return; }
  if (capAction.value === 'erase') { capAction.value = null; return; }
  capAction.value = null;
  // 6-P/6-R: a sub-threshold press-release (jitter-safe 8px) over a detected
  // window snaps that window; any real drag keeps the drawn selection. The
  // snapped selection is unconfirmed (T1) — ✓ is the only confirmation.
  if (rect.value.w < 8 && rect.value.h < 8 && downHover.value) {
    const r = downHover.value;
    rect.value = { x: r.x, y: r.y, w: r.w, h: r.h };
    confirmed.value = false;
    hoverRect.value = null;
  }
  downHover.value = null;
};

const confirmRect = async () => {
  const r = rect.value;
  if (r.w < 4 || r.h < 4) { try { await invoke('cancel_capture'); } catch (_) {} return; }
  try {
    await invoke('capture_region', {
      x: Math.round(r.x), y: Math.round(r.y), w: Math.round(r.w), h: Math.round(r.h),
      annotated: annots.value.length ? compositeRegion() : undefined,
    });
  } catch (_) { try { await invoke('cancel_capture'); } catch (_) {} }
};
const cancelRect = async () => { try { await invoke('cancel_capture'); } catch (_) {} };

// Clear stored SSH passwords from the OS keyring.
const clearTargetPassword = async () => {
  try {
    await invoke('clear_ssh_password', { user: tempTarget.value.username || '', host: tempTarget.value.host || '', port: tempTarget.value.port || null });
    tempTargetHasPassword.value = false;
    tempTarget.value.password = '';
    showToast(t('Password cleared.'));
  } catch (err) {
    showToast(`${t('Failed to clear:')} ${err}`, true);
  }
};

// Edit Custom Target
const editTarget = async (index) => {
  editingTargetIndex.value = index;
  const target = config.value.targets[index];
  tempTarget.value = { 
    ...target, 
    password: '',
    remember_password: target.remember_password !== undefined ? target.remember_password : true
  };
  tempTargetHasPassword.value = (target.type === 'ssh')
    ? await invoke('has_ssh_password', { user: target.username || '', host: target.host || '', port: target.port || null }).catch(() => false)
    : false;
  showAddTargetModal.value = true;
};

// Delete Custom Target
const deleteTarget = (index) => {
  config.value.targets.splice(index, 1);
  showToast(t('Target deleted.'));
};

// Per-target connection test state (6-H), keyed by match_pattern so it stays
// attached to the right card across deletes. 'testing' | 'ok' | 'fail'.
const targetTest = ref({});
const testTargetCard = async (tg) => {
  if (!tg || tg.type !== 'ssh' || targetTest.value[tg.match_pattern] === 'testing') return;
  targetTest.value = { ...targetTest.value, [tg.match_pattern]: 'testing' };
  try {
    await invoke('test_connection', { host: tg.host, port: tg.port || null, username: tg.username || null, password: null });
    targetTest.value = { ...targetTest.value, [tg.match_pattern]: 'ok' };
  } catch (_) {
    targetTest.value = { ...targetTest.value, [tg.match_pattern]: 'fail' };
  }
};
const targetTestTitle = (tg) => ({
  ok: t('Connected'), fail: t('Connection failed:'), testing: t('Testing...'),
}[targetTest.value[tg.match_pattern]] || '');
// A target is the default when it carries the flag (6-Q).

// Save Custom Target (add or edit)
const saveTarget = async () => {
  if (!tempTarget.value.match_pattern.trim()) {
    showToast(t('Match pattern cannot be empty.'), true);
    return;
  }

  // Password is stored in the OS keyring, never in config.toml.
  const { password, ...targetData } = { ...tempTarget.value };
  if (editingTargetIndex.value !== null) {
    config.value.targets[editingTargetIndex.value] = targetData;
  } else {
    config.value.targets.push(targetData);
  }

  let pwAction = ''; // 'stored', 'cleared', or ''
  if (targetData.type === 'ssh') {
    const user = targetData.username || '';
    const host = targetData.host || '';
    const port = targetData.port || null;
    
    if (tempTarget.value.remember_password) {
      if (password) {
        try {
          await invoke('set_ssh_password', { user, host, port, password });
          tempTargetHasPassword.value = true;
          pwAction = 'stored';
        } catch (err) {
          showToast(`${t('Saved target, but password not stored:')} ${err}`, true);
        }
      }
    } else {
      try {
        await invoke('clear_ssh_password', { user, host, port });
        tempTargetHasPassword.value = false;
        pwAction = 'cleared';
      } catch (err) {
        console.error('Failed to clear target password from keyring:', err);
      }
    }
  }

  closeTargetModal();
  if (pwAction === 'stored') {
    showToast(t('Target saved · password stored in keyring'));
  } else if (pwAction === 'cleared') {
    showToast(t('Target saved · password cleared from keyring'));
  } else {
    showToast(t('Target updated.'));
  }
};

// Close modal & reset tempTarget
const closeTargetModal = () => {
  showAddTargetModal.value = false;
  editingTargetIndex.value = null;
  tempTarget.value = {
    enabled: true,
    type: 'ssh',
    match_pattern: '',
    host: '',
    port: 22,
    username: '',
    remote_dir: '',
    local_dir: '',
    password: '',
    remember_password: true,
    is_default: false
  };
};

// Fetch initial log history and setup listener
const setupLogs = async () => {
  try {
    const history = await invoke('get_log_history');
    logs.value = history;
    scrollLogsToBottom();
  } catch (err) {
    console.error('Failed to load log history:', err);
  }

  // Listen to new log append events
  await listen('log_append', (event) => {
    logs.value.push(event.payload);
    if (logs.value.length > 200) {
      logs.value.shift();
    }
    scrollLogsToBottom();
  });
};

const scrollLogsToBottom = () => {
  nextTick(() => {
    if (logContainer.value) {
      logContainer.value.scrollTop = logContainer.value.scrollHeight;
    }
  });
};

onMounted(() => {
  const params = new URLSearchParams(window.location.search);
  if (params.get('pin') !== null) {
    // Pin-to-screen window: exact-fit image, drag to move, wheel zoom,
    // right-click / double-click close (ShareX interaction spec).
    pinMode.value = true;
    pinIdNum.value = Number(params.get('pin')) || 0;
    invoke('get_pin_image', { id: pinIdNum.value })
      .then(async (dataUrl) => {
        pinImg.value = dataUrl;
        await nextTick();
        pinBase.value = { w: window.innerWidth, h: window.innerHeight };
      })
      .catch((e) => console.error('pin image load failed:', e));
    return;
  }
  if (params.get('capture') === '1') {
    // This webview is the region-capture overlay. T6: the window is PERSISTENT
    // (hidden between captures) — each hotkey fires `capture-refresh`, which
    // reloads the frozen frame and resets session state.
    captureMode.value = true;
    listen('capture-refresh', () => { loadCaptureImage(); });
    // Capture-phase listener: fires before any child's stopPropagation (the
    // inline text editor stops keydown), and before focus quirks can eat Esc.
    window.addEventListener(
      'keydown',
      (e) => {
        // While the text editor is open it owns all keys (except what its own
        // handler defers); Esc there discards the editor, not the overlay.
        if (editingText.value) return;
      // v0.4.2 annotation shortcuts.
      if (e.ctrlKey && e.code === 'KeyZ') { e.preventDefault(); e.shiftKey ? redoAnnot() : undoAnnot(); return; }
      if (e.ctrlKey && e.code === 'KeyY') { e.preventDefault(); redoAnnot(); return; }
      if (e.key === 'Escape') {
        e.preventDefault();
        invoke('cancel_capture').catch((err) => console.error('cancel failed:', err));
      }
      // T4: Ctrl+C copies the (annotated) image and exits — Snipaste parity.
      else if (e.ctrlKey && e.code === 'KeyC' && hasRect.value) {
        e.preventDefault();
        actionCopy();
      }
      // T1: Enter is dual-purpose — confirm first, upload+inject once
      // confirmed (same two-Enter total as before).
      else if (e.key === 'Enter' && hasRect.value) {
        if (!confirmed.value) confirmSelection();
        else confirmRect();
      }
      // v0.4.1 keys — matched via e.code (physical key): with a CJK IME
      // active, keydown e.key arrives as "Process" for letter/punct keys,
      // which silently killed WASD / Shift+R / `,` / `.` on IME systems.
      else if (e.shiftKey && e.code === 'KeyR') recallHistory(0);
      else if (e.code === 'Comma') cycleHistory(-1);
      else if (e.code === 'Period') cycleHistory(1);
      else if (['KeyW', 'KeyA', 'KeyS', 'KeyD'].includes(e.code)) {
        const k = e.code.slice(3).toLowerCase();
        const dx = k === 'a' ? -1 : k === 'd' ? 1 : 0;
        const dy = k === 'w' ? -1 : k === 's' ? 1 : 0;
        invoke('nudge_cursor', { dx, dy }).catch(() => {});
      }
      // 6-S: Tab cycles the window/element candidates under the cursor.
      else if (e.key === 'Tab') { cycleCandidate(e.shiftKey ? -1 : 1); e.preventDefault(); }
      },
      { capture: true }
    );
    
    loadCaptureImage();
    return;
  }
  loadConfig();
  setupLogs();
  const overrideTheme = params.get('theme');
  if (overrideTheme && themes[overrideTheme]) config.value.theme = overrideTheme;
});
</script>

<style>
/* Apple Typography and spacing resets */
body {
  font-family: "SF Pro Text", "SF Pro Display", "Inter", system-ui, -apple-system, sans-serif;
  letter-spacing: -0.01em;
  background-color: #08080c;
}

h1, h2 {
  letter-spacing: -0.02em;
}

/* Custom styled range slider */
input[type="range"]::-webkit-slider-thumb {
  height: 16px;
  width: 16px;
  border-radius: 50%;
  background: var(--color-accent);
  cursor: pointer;
  -webkit-appearance: none;
  margin-top: -4px;
}
input[type="range"]::-webkit-slider-runnable-track {
  width: 100%;
  height: 8px;
  cursor: pointer;
  background: var(--bg-input, #020617);
  border-radius: 4px;
  border: 1px solid var(--color-input-border, #1e293b);
}</style>
