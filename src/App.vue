<template>
  <!-- Region-capture overlay (screenshot hotkey opens index.html?capture=1) -->
  <div v-if="captureMode" class="fixed inset-0 z-[9999] cursor-crosshair select-none" @mousedown="capDown" @mousemove="capMove" @mouseup="capUp">
    <img v-if="capturedImageSrc" :src="capturedImageSrc" class="absolute inset-0 w-full h-full object-cover pointer-events-none" />
    <div class="absolute top-5 left-1/2 -translate-x-1/2 text-white text-sm bg-black/70 px-4 py-1.5 rounded-full pointer-events-none shadow-lg z-[10000]">Drag to select a region · Esc to cancel</div>
    <div v-if="cap.active" :style="capRectStyle" class="absolute border-2 border-[var(--color-accent)] pointer-events-none z-[10000]" style="background: transparent; box-shadow: 0 0 0 9999px rgba(0,0,0,0.4)"></div>
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
            <p class="text-xs text-[var(--color-text-secondary)]">Settings v0.3.5</p>
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
            General Settings
          </button>

          <button 
            @click="activeTab = 'hosts'"
            :class="['w-full flex items-center gap-3 px-4 py-3 rounded-xl transition-all duration-200 font-medium text-sm', activeTab === 'hosts' ? 'bg-[var(--color-accent)] text-white shadow-sm shadow-[var(--color-accent)]/15' : 'text-[var(--color-text-secondary)] hover:bg-white/[0.02] hover:text-[var(--color-text-primary)]']"
          >
            <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
            </svg>
            Hosts & Targets
          </button>

          <button 
            @click="activeTab = 'logs'"
            :class="['w-full flex items-center gap-3 px-4 py-3 rounded-xl transition-all duration-200 font-medium text-sm', activeTab === 'logs' ? 'bg-[var(--color-accent)] text-white shadow-sm shadow-[var(--color-accent)]/15' : 'text-[var(--color-text-secondary)] hover:bg-white/[0.02] hover:text-[var(--color-text-primary)]']"
          >
            <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
            </svg>
            System Logs
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
              <h2 class="text-2xl font-bold tracking-tight text-[var(--color-text-primary)]">General Settings</h2>
              <p class="text-sm text-[var(--color-text-secondary)]">Configure global screenshot format, hotkeys, and injection preferences.</p>
            </div>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <!-- Left Card -->
            <div class="bg-[var(--bg-card)] backdrop-blur-2xl border border-[var(--color-border)] rounded-2xl p-6 space-y-4 shadow-[0_8px_32px_rgba(0,0,0,0.37)]">
              <h3 class="text-sm font-semibold uppercase text-[var(--color-text-secondary)] tracking-wider">Image Config</h3>
              
              <div>
                <label class="block text-xs font-semibold text-[var(--color-text-secondary)] mb-1">Output Format</label>
                <select v-model="config.output_format" class="w-full bg-[var(--bg-input)] border border-[var(--color-input-border)] rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-accent)] text-[var(--color-text-primary)]">
                  <option value="markdown">Markdown (![image](path))</option>
                  <option value="html">HTML (&lt;img src="path" /&gt;)</option>
                  <option value="raw">Raw Path</option>
                  <option value="base64">Inline Base64 Data URI</option>
                </select>
              </div>

              <div>
                <div class="flex justify-between text-xs font-semibold text-[var(--color-text-secondary)] mb-1">
                  <span>Compression Quality</span>
                  <span class="text-[var(--color-accent)]">{{ config.compress_quality }}%</span>
                </div>
                <input type="range" min="10" max="100" v-model.number="config.compress_quality" class="w-full accent-[var(--color-accent)] bg-[var(--bg-input)]" />
              </div>

              <div>
                <label class="block text-xs font-semibold text-[var(--color-text-secondary)] mb-1">Max Dimension (Pixels)</label>
                <input type="number" v-model.number="config.max_dimension" placeholder="No Limit" class="w-full bg-[var(--bg-input)] border border-[var(--color-input-border)] rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-accent)] text-[var(--color-text-primary)]" />
              </div>
            </div>

            <!-- Right Card -->
            <div class="bg-[var(--bg-card)] backdrop-blur-2xl border border-[var(--color-border)] rounded-2xl p-6 space-y-4 shadow-[0_8px_32px_rgba(0,0,0,0.37)]">
              <h3 class="text-sm font-semibold uppercase text-[var(--color-text-secondary)] tracking-wider">System Integration</h3>

              <div>
                <label class="block text-xs font-semibold text-[var(--color-text-secondary)] mb-1">
                  Paste Hotkey 
                  <span v-if="recordingHotkey" class="text-[var(--color-accent)] font-bold ml-1 animate-pulse">(Recording...)</span>
                  <span v-else class="text-[var(--color-text-secondary)]/80 normal-case font-normal ml-1">(click & press keys)</span>
                </label>
                <div class="flex gap-2">
                  <input type="text" readonly :value="config.global_hotkey" @focus="recordingHotkey = true" @blur="recordingHotkey = false" @keydown="recordHotkeyKeydown" :class="['flex-1 bg-[var(--bg-input)] border rounded-xl px-3 py-2 text-sm focus:outline-none text-[var(--color-text-primary)] font-mono cursor-pointer transition-all', recordingHotkey ? 'border-[var(--color-accent)] shadow-[0_0_0_2px_rgba(41,151,255,0.2)]' : 'border-[var(--color-input-border)] focus:border-[var(--color-accent)]']" />
                  <button type="button" @click="config.global_hotkey = 'Alt+V'" class="px-3 py-2 text-xs font-medium bg-[var(--bg-button)] hover:bg-[var(--bg-button-hover)] text-[var(--color-text-secondary)] rounded-xl transition-colors border border-[var(--color-input-border)]">Reset</button>
                </div>
              </div>
              <div>
                <label class="block text-xs font-semibold text-[var(--color-text-secondary)] mb-1">
                  Screenshot Hotkey 
                  <span v-if="recordingShot" class="text-[var(--color-accent)] font-bold ml-1 animate-pulse">(Recording...)</span>
                  <span v-else class="text-[var(--color-text-secondary)]/80 normal-case font-normal ml-1">(region capture)</span>
                </label>
                <div class="flex gap-2">
                  <input type="text" readonly :value="config.screenshot_hotkey" @focus="recordingShot = true" @blur="recordingShot = false" @keydown="recordShotKeydown" :class="['flex-1 bg-[var(--bg-input)] border rounded-xl px-3 py-2 text-sm focus:outline-none text-[var(--color-text-primary)] font-mono cursor-pointer transition-all', recordingShot ? 'border-[var(--color-accent)] shadow-[0_0_0_2px_rgba(41,151,255,0.2)]' : 'border-[var(--color-input-border)] focus:border-[var(--color-accent)]']" />
                  <button type="button" @click="config.screenshot_hotkey = 'Alt+Shift+S'" class="px-3 py-2 text-xs font-medium bg-[var(--bg-button)] hover:bg-[var(--bg-button-hover)] text-[var(--color-text-secondary)] rounded-xl transition-colors border border-[var(--color-input-border)]">Reset</button>
                </div>
              </div>

              <div>
                <label class="block text-xs font-semibold text-[var(--color-text-secondary)] mb-1">Injection Mode</label>
                <select v-model="config.injection_mode" class="w-full bg-[var(--bg-input)] border border-[var(--color-input-border)] rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-accent)] text-[var(--color-text-primary)]">
                  <option value="direct">Direct Native Keystrokes (Bypasses IME)</option>
                  <option value="swap">Quick Clipboard Swap & Paste</option>
                </select>
              </div>

              <div class="flex items-center justify-between py-1">
                <div>
                  <span class="block text-sm font-medium text-[var(--color-text-primary)]">Wrap in Single Quotes</span>
                  <span class="block text-xs text-[var(--color-text-secondary)]">Wrap generated link in 'quotes'</span>
                </div>
                <label class="relative inline-flex items-center cursor-pointer">
                  <input type="checkbox" v-model="config.wrap_single_quotes" class="sr-only peer" />
                  <div class="w-11 h-6 bg-[var(--bg-toggle)] rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-[var(--color-toggle-knob)] after:border-[var(--color-toggle-knob)] after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-[var(--color-accent)]"></div>
                </label>
              </div>

              <div class="flex items-center justify-between py-1">
                <div>
                  <span class="block text-sm font-medium text-[var(--color-text-primary)]">Launch on Boot</span>
                  <span class="block text-xs text-[var(--color-text-secondary)]">Start img2cli automatically</span>
                </div>
                <label class="relative inline-flex items-center cursor-pointer">
                  <input type="checkbox" v-model="config.launch_on_boot" class="sr-only peer" />
                  <div class="w-11 h-6 bg-[var(--bg-toggle)] rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-[var(--color-toggle-knob)] after:border-[var(--color-toggle-knob)] after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-[var(--color-accent)]"></div>
                </label>
              </div>

              <div class="flex items-center justify-between py-1">
                <div>
                  <span class="block text-sm font-medium text-[var(--color-text-primary)]">Enable Desktop Notifications</span>
                  <span class="block text-xs text-[var(--color-text-secondary)]">Show tips on screenshot success</span>
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
            <h3 class="text-sm font-semibold uppercase text-[var(--color-text-secondary)] tracking-wider">Interface Theme</h3>
            <div class="flex items-center gap-3">
              <span class="w-5 h-5 rounded-full border border-[var(--color-border)] shrink-0 shadow-inner" :style="{ backgroundColor: currentTheme.colorAccent }" :title="'Accent: ' + currentTheme.colorAccent"></span>
              <select v-model="config.theme" class="flex-1 bg-[var(--bg-input)] border border-[var(--color-input-border)] rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-accent)] text-[var(--color-text-primary)]">
                <option v-for="(tOpts, name) in themes" :key="name" :value="name">{{ themeLabel(name) }}</option>
              </select>
            </div>
          </div>

          <!-- Save Directory Config -->
          <div class="bg-[var(--bg-card)] border border-[var(--color-border)] rounded-2xl p-6 space-y-4 shadow-[0_8px_32px_rgba(0,0,0,0.37)]">
            <h3 class="text-sm font-semibold uppercase text-[var(--color-text-secondary)] tracking-wider">Advanced Paths</h3>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div>
                <label class="block text-xs font-semibold text-[var(--color-text-secondary)] mb-1">Local Temporary Directory</label>
                <input type="text" v-model="config.save_dir" placeholder="Default (Temp Dir/img2cli)" class="w-full bg-[var(--bg-input)] border border-[var(--color-input-border)] rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-accent)] text-[var(--color-text-primary)]" />
              </div>
              <div>
                <label class="block text-xs font-semibold text-[var(--color-text-secondary)] mb-1">Clean Expired Image Files (Days)</label>
                <input type="number" v-model.number="config.clean_keep_days" class="w-full bg-[var(--bg-input)] border border-[var(--color-input-border)] rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-accent)] text-[var(--color-text-primary)]" />
              </div>
            </div>
          </div>

          <div class="flex justify-end pt-2">
            <button @click="saveSettings" class="flex items-center gap-2 bg-[var(--color-accent)] hover:bg-[var(--color-accent-hover)] text-white px-6 py-2.5 rounded-full font-semibold shadow-sm shadow-[var(--color-accent)]/15 active:scale-[0.98] transition-all duration-150 text-sm">
              Save Settings
            </button>
          </div>
        </div>

        <!-- Hosts & Targets Tab -->
        <div v-if="activeTab === 'hosts'" class="space-y-6">
          <div>
            <h2 class="text-2xl font-bold tracking-tight text-[var(--color-text-primary)]">Hosts & Targets</h2>
            <p class="text-sm text-[var(--color-text-secondary)]">Configure remote SSH servers and local workspace directory routing.</p>
          </div>

          <!-- Default SSH Config -->
          <div class="bg-[var(--bg-card)] backdrop-blur-2xl border border-[var(--color-border)] rounded-2xl p-6 space-y-4 shadow-[0_8px_32px_rgba(0,0,0,0.37)]">
            <div class="flex items-center justify-between border-b border-[var(--color-input-border)] pb-3">
              <label class="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" v-model="config.ssh.enabled" class="sr-only peer" />
                <div class="w-11 h-6 bg-[var(--bg-toggle)] rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-[var(--color-toggle-knob)] after:border-[var(--color-toggle-knob)] after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-[var(--color-accent)]"></div>
              </label>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <label class="block text-xs font-semibold text-[var(--color-text-secondary)] mb-1">Host Name</label>
                <input type="text" v-model="config.ssh.match_pattern" :disabled="!config.ssh.enabled" placeholder="e.g. My GPU Server" class="w-full bg-[var(--bg-input)] border border-[var(--color-input-border)] rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-accent)] text-[var(--color-text-primary)] disabled:opacity-50" />
              </div>
              <div>
                <label class="block text-xs font-semibold text-[var(--color-text-secondary)] mb-1">Host IP / Address</label>
                <input type="text" v-model="config.ssh.host" :disabled="!config.ssh.enabled" class="w-full bg-[var(--bg-input)] border border-[var(--color-input-border)] rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-accent)] text-[var(--color-text-primary)] disabled:opacity-50" />
              </div>
              <div>
                <label class="block text-xs font-semibold text-[var(--color-text-secondary)] mb-1">Port</label>
                <input type="number" v-model.number="config.ssh.port" :disabled="!config.ssh.enabled" class="w-full bg-[var(--bg-input)] border border-[var(--color-input-border)] rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-accent)] text-[var(--color-text-primary)] disabled:opacity-50" />
              </div>
              <div>
                <label class="block text-xs font-semibold text-[var(--color-text-secondary)] mb-1">Username</label>
                <input type="text" v-model="config.ssh.username" :disabled="!config.ssh.enabled" class="w-full bg-[var(--bg-input)] border border-[var(--color-input-border)] rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-accent)] text-[var(--color-text-primary)] disabled:opacity-50" />
              </div>
              <div class="md:col-span-2">
                <label class="block text-xs font-semibold text-[var(--color-text-secondary)] mb-1">Remote Copy Destination Folder</label>
                <input type="text" v-model="config.ssh.remote_dir" :disabled="!config.ssh.enabled" class="w-full bg-[var(--bg-input)] border border-[var(--color-input-border)] rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-accent)] text-[var(--color-text-primary)] disabled:opacity-50" />
              </div>
              <div>
                <label class="block text-xs font-semibold text-[var(--color-text-secondary)] mb-1">Password <span class="text-[var(--color-text-secondary)]/80 normal-case font-normal">(OS keyring)</span></label>
                <input type="password" v-model="defaultPassword" :disabled="!config.ssh.enabled" :placeholder="defaultHasPassword ? '●●●●●● (saved) — type a new one to update' : 'blank: uses your SSH key (~/.ssh)'" class="w-full bg-[var(--bg-input)] border border-[var(--color-input-border)] rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-accent)] text-[var(--color-text-primary)] disabled:opacity-50" />
                <div class="flex items-center gap-2 mt-1.5">
                  <input type="checkbox" id="default-remember-pwd" v-model="config.ssh.remember_password" :disabled="!config.ssh.enabled" class="accent-[var(--color-accent)] rounded bg-[var(--bg-input)] border-[var(--color-input-border)]" />
                  <label for="default-remember-pwd" class="text-xs font-medium text-[var(--color-text-secondary)] cursor-pointer select-none">Remember Password (OS Keyring)</label>
                </div>
                <div class="text-[11px] mt-1 flex items-center gap-2">
                  <template v-if="defaultHasPassword">
                    <span class="text-emerald-400">✓ Password saved (keyring)</span>
                    <button type="button" @click="clearDefaultPassword" :disabled="!config.ssh.enabled" class="text-red-400/80 hover:text-red-400 underline disabled:opacity-50">clear</button>
                  </template>
                  <span v-else class="text-[var(--color-text-secondary)]">No password → will use your SSH key (~/.ssh)</span>
                </div>
              </div>
            </div>

            <div class="flex justify-end pt-2">
              <button 
                @click="checkSSHConnection" 
                :disabled="!config.ssh.enabled || testingConnection"
                class="bg-[var(--bg-button)] hover:bg-[var(--bg-button-hover)] text-[var(--color-text-primary)] font-semibold px-4 py-2 rounded-xl text-xs active:scale-[0.98] transition-all disabled:opacity-50 flex items-center gap-2"
              >
                <span v-if="testingConnection" class="w-3 h-3 border-2 border-[var(--color-text-secondary)] border-t-transparent rounded-full animate-spin"></span>
                {{ testingConnection ? 'Testing...' : 'Test Connection' }}
              </button>
            </div>
          </div>

          <!-- Dynamic Targets List -->
          <div class="bg-[var(--bg-card)] backdrop-blur-2xl border border-[var(--color-border)] rounded-2xl p-6 space-y-4 shadow-[0_8px_32px_rgba(0,0,0,0.37)]">
            <div class="flex items-center justify-between border-b border-[var(--color-input-border)] pb-3">
              <h3 class="text-sm font-semibold uppercase text-[var(--color-text-secondary)] tracking-wider">Dynamic Router Targets</h3>
              <div class="flex items-center gap-2">
                <button
                  @click="openSshLoader"
                  :disabled="loadingSsh"
                  class="bg-white/5 hover:bg-white/10 border border-[var(--color-border)] text-[var(--color-text-primary)] font-semibold px-3 py-1.5 rounded-xl text-xs flex items-center gap-1 active:scale-[0.98] transition-all disabled:opacity-50"
                >
                  <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
                  </svg>
                  Load SSH Config
                </button>
                <button
                  @click="showAddTargetModal = true"
                  class="bg-[var(--color-accent)] hover:bg-[var(--color-accent-hover)] text-white font-semibold px-3 py-1.5 rounded-xl text-xs flex items-center gap-1 active:scale-[0.98] transition-all"
                >
                  <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
                  </svg>
                  Add Target
                </button>
              </div>
            </div>

            <!-- Targets Table -->
            <div>
              <table class="w-full text-left border-collapse">
                <thead>
                  <tr class="border-b border-[var(--color-input-border)] text-xs font-semibold text-[var(--color-text-secondary)]">
                    <th class="w-16 py-3 px-4 text-center">Status</th>
                    <th class="w-40 py-3 px-4 text-left">Host Name / Alias</th>
                    <th class="w-24 py-3 px-4 text-center">Type</th>
                    <th class="py-3 px-4 text-left">Details</th>
                    <th class="w-56 py-3 px-4 text-center">Actions</th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-[var(--color-border)] text-sm">
                  <tr v-for="(target, idx) in (config.targets || [])" :key="idx" class="hover:bg-[var(--color-text-primary)]/[0.04]">
                    <td class="py-3 px-4 text-center">
                      <label class="relative inline-flex items-center cursor-pointer">
                        <input type="checkbox" v-model="target.enabled" class="sr-only peer" />
                        <div class="w-9 h-5 bg-[var(--bg-toggle)] rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-[var(--color-toggle-knob)] after:border-[var(--color-toggle-knob)] after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-[var(--color-accent)]"></div>
                      </label>
                    </td>
                    <td class="py-3 px-4 font-semibold text-[var(--color-text-primary)] max-w-[10rem] truncate">{{ target.match_pattern }}</td>
                    <td class="py-3 px-4 text-center">
                      <span :class="['px-2 py-0.5 rounded-md text-xs font-semibold uppercase', target.type === 'ssh' ? 'bg-[var(--color-accent)]/10 text-[var(--color-accent)] border border-[var(--color-accent)]/25' : 'bg-[var(--color-text-secondary)]/10 text-[var(--color-text-secondary)] border border-[var(--color-text-secondary)]/25']">
                        {{ target.type }}
                      </span>
                    </td>
                    <td class="py-3 px-4 text-xs text-[var(--color-text-secondary)] max-w-[28rem] truncate">
                      <span v-if="target.type === 'ssh'" class="block truncate" :title="`${target.username}@${target.host}:${target.remote_dir}`">{{ target.username }}@{{ target.host }}:{{ target.remote_dir }}</span>
                      <span v-else class="block truncate" :title="target.local_dir">{{ target.local_dir }}</span>
                    </td>
                    <td class="py-3 px-4 text-center">
                      <div class="flex items-center justify-center gap-1.5">
                        <button v-if="target.type === 'ssh'" @click="setAsDefault(idx)" class="px-1.5 py-0.5 rounded-md text-[11px] font-semibold bg-[var(--color-accent)]/10 text-[var(--color-accent)] border border-[var(--color-accent)]/25 hover:bg-[var(--color-accent)]/20 transition-colors">Set Default</button>
                        <button @click="editTarget(idx)" class="px-1.5 py-0.5 rounded-md text-[11px] font-semibold bg-[var(--color-text-secondary)]/10 text-[var(--color-text-secondary)] border border-[var(--color-text-secondary)]/25 hover:bg-[var(--color-text-secondary)]/20 transition-colors">Edit</button>
                        <button @click="deleteTarget(idx)" class="px-1.5 py-0.5 rounded-md text-[11px] font-semibold bg-red-500/10 text-red-400 border border-red-500/25 hover:bg-red-500/20 transition-colors">Delete</button>
                      </div>
                    </td>
                  </tr>
                  <tr v-if="!(config.targets || []).length">
                    <td colspan="5" class="py-6 text-center text-[var(--color-text-secondary)] text-xs">No routing targets configured. Clipboard uploads will fallback to default host.</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>

          <div class="flex justify-end pt-2">
            <button @click="saveSettings" class="flex items-center gap-2 bg-[var(--color-accent)] hover:bg-[var(--color-accent-hover)] text-white px-6 py-2.5 rounded-full font-semibold shadow-sm shadow-[var(--color-accent)]/15 active:scale-[0.98] transition-all duration-150 text-sm">
              Save Settings
            </button>
          </div>
        </div>

        <!-- System Logs Tab -->
        <div v-if="activeTab === 'logs'" class="space-y-6 flex flex-col h-[calc(100vh-8rem)]">
          <div class="flex justify-between items-center shrink-0">
            <div>
              <h2 class="text-2xl font-bold tracking-tight text-[var(--color-text-primary)]">System Logs</h2>
              <p class="text-sm text-[var(--color-text-secondary)]">Real-time daemon events and screenshot processing logs.</p>
            </div>
            <button @click="logs = []" class="bg-[var(--bg-button)] hover:bg-[var(--bg-button-hover)] text-[var(--color-text-primary)] font-semibold px-3 py-1.5 rounded-xl text-xs active:scale-[0.98] transition-all">
              Clear Logs
            </button>
          </div>

          <div class="flex-1 bg-[var(--bg-input)] border border-[var(--color-input-border)] rounded-2xl p-4 overflow-y-auto font-mono text-xs text-[var(--color-text-secondary)] space-y-1.5 shadow-inner" ref="logContainer">
            <div v-for="(log, idx) in logs" :key="idx" class="whitespace-pre-wrap leading-relaxed">
              <span class="text-[var(--color-text-secondary)]/80 select-none">[{{ idx + 1 }}]</span> {{ log }}
            </div>
            <div v-if="!logs.length" class="text-[var(--color-text-secondary)]/80 text-center py-12">No logs loaded. Press global hotkey to trigger daemon activity.</div>
          </div>
        </div>

      </main>
    </div>

    <!-- Add/Edit Target Modal -->
    <div v-if="showAddTargetModal" class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50">
      <div class="bg-[var(--bg-card)] backdrop-blur-2xl border border-[var(--color-border)] rounded-2xl max-w-lg w-full p-6 space-y-4 shadow-2xl">
        <h3 class="text-lg font-bold text-[var(--color-text-primary)]">{{ editingTargetIndex !== null ? 'Edit Router Target' : 'Add Router Target' }}</h3>
        
        <div class="space-y-3">
          <div class="grid grid-cols-2 gap-3">
            <div>
              <label class="block text-xs font-semibold text-[var(--color-text-secondary)] mb-1">Target Type</label>
              <select v-model="tempTarget.type" class="w-full bg-[var(--bg-input)] border border-[var(--color-input-border)] rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-accent)] text-[var(--color-text-primary)]">
                <option value="ssh">SSH (Remote Server)</option>
                <option value="local">Local Folder</option>
              </select>
            </div>
            <div>
              <label class="block text-xs font-semibold text-[var(--color-text-secondary)] mb-1">Host Name / Alias</label>
              <input type="text" v-model="tempTarget.match_pattern" placeholder="e.g. GPU-90, WSL" class="w-full bg-[var(--bg-input)] border border-[var(--color-input-border)] rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-accent)] text-[var(--color-text-primary)]" />
            </div>
          </div>

          <!-- SSH Target Fields -->
          <div v-if="tempTarget.type === 'ssh'" class="grid grid-cols-2 gap-3">
            <div>
              <label class="block text-xs font-semibold text-[var(--color-text-secondary)] mb-1">Host IP / Address</label>
              <input type="text" v-model="tempTarget.host" class="w-full bg-[var(--bg-input)] border border-[var(--color-input-border)] rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-accent)] text-[var(--color-text-primary)]" />
            </div>
            <div>
              <label class="block text-xs font-semibold text-[var(--color-text-secondary)] mb-1">Port</label>
              <input type="number" v-model.number="tempTarget.port" class="w-full bg-[var(--bg-input)] border border-[var(--color-input-border)] rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-accent)] text-[var(--color-text-primary)]" />
            </div>
            <div>
              <label class="block text-xs font-semibold text-[var(--color-text-secondary)] mb-1">Username</label>
              <input type="text" v-model="tempTarget.username" class="w-full bg-[var(--bg-input)] border border-[var(--color-input-border)] rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-accent)] text-[var(--color-text-primary)]" />
            </div>
            <div>
              <label class="block text-xs font-semibold text-[var(--color-text-secondary)] mb-1">Remote Copy Destination Folder</label>
              <input type="text" v-model="tempTarget.remote_dir" class="w-full bg-[var(--bg-input)] border border-[var(--color-input-border)] rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-accent)] text-[var(--color-text-primary)]" />
            </div>
            <div>
              <label class="block text-xs font-semibold text-[var(--color-text-secondary)] mb-1">Password <span class="text-[var(--color-text-secondary)]/80 normal-case font-normal">(OS keyring)</span></label>
              <input type="password" v-model="tempTarget.password" :placeholder="tempTargetHasPassword ? '●●●●●● (saved) — type a new one to update' : 'blank: uses your SSH key (~/.ssh)'" class="w-full bg-[var(--bg-input)] border border-[var(--color-input-border)] rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-accent)] text-[var(--color-text-primary)]" />
              <div class="flex items-center gap-2 mt-1.5">
                <input type="checkbox" id="target-remember-pwd" v-model="tempTarget.remember_password" class="accent-[var(--color-accent)] rounded bg-[var(--bg-input)] border-[var(--color-input-border)]" />
                <label for="target-remember-pwd" class="text-xs font-medium text-[var(--color-text-secondary)] cursor-pointer select-none">Remember Password (OS Keyring)</label>
              </div>
              <div class="text-[11px] mt-1 flex items-center gap-2">
                <template v-if="tempTargetHasPassword">
                  <span class="text-emerald-400">✓ Password saved (keyring)</span>
                  <button type="button" @click="clearTargetPassword" class="text-red-400/80 hover:text-red-400 underline">clear</button>
                </template>
                <span v-else class="text-[var(--color-text-secondary)]">No password → will use your SSH key (~/.ssh)</span>
              </div>
            </div>
          </div>

          <!-- Local Target Fields -->
          <div v-if="tempTarget.type === 'local'">
            <label class="block text-xs font-semibold text-[var(--color-text-secondary)] mb-1">Local Copy Destination Folder</label>
            <input type="text" v-model="tempTarget.local_dir" placeholder="e.g. C:\users\docs\images" class="w-full bg-[var(--bg-input)] border border-[var(--color-input-border)] rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-accent)] text-[var(--color-text-primary)]" />
          </div>
        </div>

        <div class="flex justify-end gap-3 pt-3 border-t border-[var(--color-input-border)]">
          <button @click="closeTargetModal" class="bg-[var(--bg-button)] hover:bg-[var(--bg-button-hover)] text-[var(--color-text-primary)] px-4 py-2 rounded-xl text-xs font-semibold">Cancel</button>
          <button @click="saveTarget" class="bg-[var(--color-accent)] hover:bg-[var(--color-accent-hover)] text-white px-4 py-2 rounded-xl text-xs font-semibold">Save</button>
        </div>
      </div>
    </div>

    <!-- SSH Config Loader Modal -->
    <div v-if="showSshModal" class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50">
      <div class="bg-[var(--bg-card)] backdrop-blur-2xl border border-[var(--color-border)] rounded-2xl max-w-lg w-full p-6 space-y-4 shadow-2xl">
        <h3 class="text-lg font-bold text-[var(--color-text-primary)]">Load OpenSSH config</h3>
        <div class="flex items-center gap-2">
          <input type="text" v-model="sshConfigPath" placeholder="~/.ssh/config" class="flex-1 bg-[var(--bg-input)]/60 border border-[var(--color-border)] rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-accent)] text-[var(--color-text-primary)] font-mono" />
          <button @click="browseSshConfig" :disabled="loadingSsh" class="bg-white/5 hover:bg-white/10 border border-[var(--color-border)] text-[var(--color-text-primary)] font-semibold px-3 py-2 rounded-xl text-xs disabled:opacity-50 whitespace-nowrap">Browse…</button>
          <button @click="openSshLoader" :disabled="loadingSsh" class="bg-white/5 hover:bg-white/10 border border-[var(--color-border)] text-[var(--color-text-primary)] font-semibold px-3 py-2 rounded-xl text-xs disabled:opacity-50 whitespace-nowrap">Load</button>
        </div>
        <input type="text" v-model="sshSearch" placeholder="Search hosts (alias / host / user)..." class="w-full bg-[var(--bg-input)]/60 border border-[var(--color-border)] rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-[var(--color-accent)] text-[var(--color-text-primary)]" />
        <div class="max-h-72 overflow-y-auto space-y-1 pr-1">
          <label v-for="{ h, i } in filteredSshHosts" :key="i" class="flex items-center gap-3 px-3 py-2 rounded-lg hover:bg-white/5 cursor-pointer">
            <input type="checkbox" v-model="sshSelected[i]" class="accent-[var(--color-accent)] w-4 h-4" />
            <div class="flex-1 min-w-0">
              <div class="text-sm font-semibold text-[var(--color-text-primary)] truncate">{{ h.alias }}</div>
              <div class="text-xs text-[var(--color-text-secondary)] truncate font-mono">{{ h.username }}@{{ h.host }}:{{ h.port }}</div>
            </div>
          </label>
          <div v-if="!filteredSshHosts.length" class="text-[var(--color-text-secondary)] text-center py-8 text-sm">No hosts found.</div>
        </div>
        <div class="flex items-center justify-between pt-3 border-t border-[var(--color-border)]">
          <button @click="toggleAllSsh(true)" class="text-xs font-semibold text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)] transition-colors">Select All</button>
          <div class="flex gap-3">
            <button @click="closeSshModal" class="bg-white/5 hover:bg-white/10 text-[var(--color-text-primary)] px-4 py-2 rounded-xl text-xs font-semibold">Cancel</button>
            <button @click="importSshSelected" class="bg-[var(--color-accent)] hover:bg-[var(--color-accent-hover)] text-white px-4 py-2 rounded-xl text-xs font-semibold">Import Selected</button>
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
import { open as openDialog } from '@tauri-apps/plugin-dialog';

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
  wrap_single_quotes: true,
  launch_on_boot: true,
  enable_notifications: true,
  global_hotkey: 'Alt+V',
  screenshot_hotkey: 'Alt+Shift+S',
  upload_strategy: 'eager',
  injection_mode: 'direct',
  clean_keep_days: 1,
  theme: 'apple-dark',
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
    bgApp: '#282a36',
    bgSidebar: 'rgba(33, 34, 44, 0.6)',
    bgCard: 'rgba(68, 71, 90, 0.4)',
    colorBorder: 'rgba(98, 114, 164, 0.3)',
    colorAccent: '#bd93f9',
    colorAccentHover: '#ff79c6',
    colorAccentDim: 'rgba(189, 147, 249, 0.1)',
    textPrimary: '#f8f8f2',
    textSecondary: '#6272a4',
    bgInput: '#1e1f29',
    colorInputBorder: '#44475a',
    bgToggle: 'rgba(98,114,164,0.30)', colorToggleKnob: '#f8f8f2', bgButton: '#44475a', bgButtonHover: '#5a5f78'
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
  const t = config.value?.theme || 'apple-dark';
  return themes[t] || themes['apple-dark'];
});

// Pretty label for the theme <select> options ("apple-dark" -> "Apple Dark").
const themeLabel = (name) => name.split('-').map((w) => w[0].toUpperCase() + w.slice(1)).join(' ');

// Logs Container & History
const logs = ref([]);
const logContainer = ref(null);

// SSH Testing State
const testingConnection = ref(false);

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

// Password for the default SSH host (stored in OS keyring, NOT in config).
const defaultPassword = ref('');
const defaultHasPassword = ref(false);
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
    showToast(`Failed to load SSH config: ${err}`, true);
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
    showToast(`Failed to open file dialog: ${err}`, true);
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
    const exists = config.value.targets.some(t => 
      t.type === 'ssh' && 
      t.match_pattern.toLowerCase() === h.alias.toLowerCase()
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
    showToast(`Imported ${added} host(s), skipped ${skipped} duplicate(s) as router targets.`);
  } else {
    showToast(`Imported ${added} host(s) as router targets.`);
  }
};

// Copy a router target's SSH host into the default host configuration
const setAsDefault = async (index) => {
  const t = config.value.targets[index];
  if (!t || t.type !== 'ssh') return;
  config.value.ssh = {
    enabled: true,
    host: t.host || '',
    port: t.port || 22,
    username: t.username || '',
    remote_dir: t.remote_dir || config.value.ssh?.remote_dir || '/tmp/img2cli',
    match_pattern: t.match_pattern || '',
    remember_password: t.remember_password !== undefined ? t.remember_password : true
  };

  // Clear current password input to avoid inserting password text across hosts
  defaultPassword.value = '';

  // Check if keyring already contains password for this host to update UI status indicator
  try {
    defaultHasPassword.value = await invoke('has_ssh_password', {
      user: t.username || '',
      host: t.host || '',
      port: t.port || null
    });
  } catch (_) {
    defaultHasPassword.value = false;
  }

  showToast(`Set "${t.match_pattern}" as the default SSH host.`);
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
      data.targets.forEach(t => {
        if (t.remember_password === undefined) {
          t.remember_password = true; // default to true if missing
        }
      });
    }
    config.value = data;
    lastSavedGlobalHotkey.value = data.global_hotkey || '';
    lastSavedScreenshotHotkey.value = data.screenshot_hotkey || '';
    if (data.ssh && data.ssh.host) {
      try {
        defaultHasPassword.value = await invoke('has_ssh_password', {
          user: data.ssh.username || '',
          host: data.ssh.host,
          port: data.ssh.port || null
        });
      } catch (_) { /* ignore */ }
    }
  } catch (err) {
    showToast(`Failed to load configuration: ${err}`, true);
  }
};

// Save Configurations
const saveSettings = async () => {
  try {
    // If remember_password is true, save password to system keyring.
    // If remember_password is false, delete password from system keyring.
    if (config.value.ssh) {
      const user = config.value.ssh.username || '';
      const host = config.value.ssh.host || '';
      const port = config.value.ssh.port || null;
      
      if (config.value.ssh.remember_password) {
        if (defaultPassword.value) {
          await invoke('set_ssh_password', { user, host, port, password: defaultPassword.value });
          defaultHasPassword.value = true;
          defaultPassword.value = '';
        }
      } else {
        await invoke('clear_ssh_password', { user, host, port });
        defaultHasPassword.value = false;
        defaultPassword.value = '';
      }
    }
    await invoke('save_config', { config: config.value });
    lastSavedGlobalHotkey.value = config.value.global_hotkey;
    lastSavedScreenshotHotkey.value = config.value.screenshot_hotkey;
    showToast('Settings saved successfully!');
  } catch (err) {
    showToast(`Failed to save settings: ${err}`, true);
  }
};

// Check SSH connection
const checkSSHConnection = async () => {
  testingConnection.value = true;
  try {
    const res = await invoke('test_connection', {
      host: config.value.ssh.host,
      port: config.value.ssh.port || null,
      username: config.value.ssh.username || null,
      password: defaultPassword.value || null
    });
    showToast(res);
  } catch (err) {
    showToast(`Connection failed: ${err}`, true);
  } finally {
    testingConnection.value = false;
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
const captureMode = ref(false);
const capturedImageSrc = ref('');
const cap = ref({ active: false, x0: 0, y0: 0, x1: 0, y1: 0 });
const capRectStyle = computed(() => ({
  left: Math.min(cap.value.x0, cap.value.x1) + 'px',
  top: Math.min(cap.value.y0, cap.value.y1) + 'px',
  width: Math.abs(cap.value.x1 - cap.value.x0) + 'px',
  height: Math.abs(cap.value.y1 - cap.value.y0) + 'px'
}));
const capDown = (e) => { cap.value = { active: true, x0: e.clientX, y0: e.clientY, x1: e.clientX, y1: e.clientY }; };
const capMove = (e) => { if (cap.value.active) { cap.value.x1 = e.clientX; cap.value.y1 = e.clientY; } };
const capUp = async (e) => {
  cap.value.active = false;
  const x = Math.round(Math.min(cap.value.x0, e.clientX));
  const y = Math.round(Math.min(cap.value.y0, e.clientY));
  const w = Math.round(Math.abs(e.clientX - cap.value.x0));
  const h = Math.round(Math.abs(e.clientY - cap.value.y0));
  if (w < 4 || h < 4) { try { await invoke('cancel_capture'); } catch (_) {} return; }
  try { await invoke('capture_region', { x, y, w, h }); } catch (_) { try { await invoke('cancel_capture'); } catch (_) {} }
};

// Clear stored SSH passwords from the OS keyring.
const clearDefaultPassword = async () => {
  if (!config.value.ssh) return;
  try {
    await invoke('clear_ssh_password', { user: config.value.ssh.username || '', host: config.value.ssh.host || '', port: config.value.ssh.port || null });
    defaultHasPassword.value = false;
    defaultPassword.value = '';
    showToast('Password cleared.');
  } catch (err) {
    showToast(`Failed to clear: ${err}`, true);
  }
};
const clearTargetPassword = async () => {
  try {
    await invoke('clear_ssh_password', { user: tempTarget.value.username || '', host: tempTarget.value.host || '', port: tempTarget.value.port || null });
    tempTargetHasPassword.value = false;
    tempTarget.value.password = '';
    showToast('Password cleared.');
  } catch (err) {
    showToast(`Failed to clear: ${err}`, true);
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
  showToast('Target deleted.');
};

// Save Custom Target (add or edit)
const saveTarget = async () => {
  if (!tempTarget.value.match_pattern.trim()) {
    showToast('Match pattern cannot be empty.', true);
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
          showToast(`Saved target, but password not stored: ${err}`, true);
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
    showToast('Target saved · password stored in keyring');
  } else if (pwAction === 'cleared') {
    showToast('Target saved · password cleared from keyring');
  } else {
    showToast('Target updated.');
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
    remember_password: true
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
  if (params.get('capture') === '1') {
    // This webview is the region-capture overlay (loaded by the screenshot hotkey).
    captureMode.value = true;
    window.addEventListener('keydown', (e) => { if (e.key === 'Escape') invoke('cancel_capture'); });
    
    // Fetch the captured screen image from Rust memory
    invoke('get_captured_image')
      .then((src) => {
        capturedImageSrc.value = src;
      })
      .catch((e) => {
        console.error("Failed to load captured image:", e);
      });
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
  background: #020617;
  border-radius: 4px;
  border: 1px solid #1e293b;
}
</style>
