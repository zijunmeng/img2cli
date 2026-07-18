<template>
  <!-- Region-capture overlay (screenshot hotkey opens index.html?capture=1) -->
  <div v-if="captureMode" class="fixed inset-0 z-[9999] cursor-crosshair select-none" style="background: rgba(0,0,0,0.28)" @mousedown="capDown" @mousemove="capMove" @mouseup="capUp">
    <div class="absolute top-5 left-1/2 -translate-x-1/2 text-white text-sm bg-black/70 px-4 py-1.5 rounded-full pointer-events-none shadow-lg">Drag to select a region · Esc to cancel</div>
    <div v-if="cap.active" :style="capRectStyle" class="absolute border-2 border-orange-400 pointer-events-none" style="background: rgba(249,115,22,0.12); box-shadow: 0 0 0 9999px rgba(0,0,0,0.4)"></div>
  </div>
  <div v-else class="relative flex h-screen text-slate-100 font-sans overflow-hidden bg-[#0a0b1e]">
    <!-- Ambient background glows (give the frosted glass something to blur) -->
    <div class="pointer-events-none absolute inset-0 z-0 overflow-hidden">
      <div class="absolute -bottom-32 -left-24 w-[30rem] h-[30rem] rounded-full bg-orange-600/20 blur-[120px]"></div>
      <div class="absolute top-1/4 -right-24 w-[28rem] h-[28rem] rounded-full bg-fuchsia-600/15 blur-[120px]"></div>
      <div class="absolute -bottom-32 left-1/3 w-[26rem] h-[26rem] rounded-full bg-indigo-600/15 blur-[120px]"></div>
    </div>
    <!-- Sidebar -->
    <div class="relative z-10 w-64 bg-white/[0.04] backdrop-blur-2xl border-r border-white/10 flex flex-col shrink-0">
      <div>
        <div class="p-6 border-b border-white/10 flex items-center gap-3">
          <img src="./assets/logo.png" class="w-8 h-8 rounded-lg shadow-lg shadow-orange-500/10 object-contain" alt="img2cli Logo" />
          <div>
            <h1 class="text-lg font-bold bg-gradient-to-r from-orange-400 to-amber-400 bg-clip-text text-transparent">img2cli</h1>
            <p class="text-xs text-slate-500">Settings v0.3.4</p>
          </div>
        </div>

        <nav class="p-4 space-y-1">
          <button 
            @click="activeTab = 'general'"
            :class="['w-full flex items-center gap-3 px-4 py-3 rounded-xl transition-all duration-200 font-medium text-sm', activeTab === 'general' ? 'bg-gradient-to-r from-orange-500 to-amber-500 text-white shadow-md shadow-orange-500/20' : 'text-slate-400 hover:bg-slate-800/40 hover:text-slate-200']"
          >
            <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4" />
            </svg>
            General Settings
          </button>

          <button 
            @click="activeTab = 'hosts'"
            :class="['w-full flex items-center gap-3 px-4 py-3 rounded-xl transition-all duration-200 font-medium text-sm', activeTab === 'hosts' ? 'bg-gradient-to-r from-orange-500 to-amber-500 text-white shadow-md shadow-orange-500/20' : 'text-slate-400 hover:bg-slate-800/40 hover:text-slate-200']"
          >
            <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
            </svg>
            Hosts & Targets
          </button>

          <button 
            @click="activeTab = 'logs'"
            :class="['w-full flex items-center gap-3 px-4 py-3 rounded-xl transition-all duration-200 font-medium text-sm', activeTab === 'logs' ? 'bg-gradient-to-r from-orange-500 to-amber-500 text-white shadow-md shadow-orange-500/20' : 'text-slate-400 hover:bg-slate-800/40 hover:text-slate-200']"
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
              <h2 class="text-2xl font-bold tracking-tight text-white">General Settings</h2>
              <p class="text-sm text-slate-400">Configure global screenshot format, hotkeys, and injection preferences.</p>
            </div>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <!-- Left Card -->
            <div class="bg-white/[0.05] backdrop-blur-2xl border border-white/10 rounded-2xl p-6 space-y-4 shadow-[0_8px_32px_rgba(0,0,0,0.37)]">
              <h3 class="text-sm font-semibold uppercase text-slate-500 tracking-wider">Image Config</h3>
              
              <div>
                <label class="block text-xs font-semibold text-slate-400 mb-1">Output Format</label>
                <select v-model="config.output_format" class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-orange-500 text-slate-200">
                  <option value="markdown">Markdown (![image](path))</option>
                  <option value="html">HTML (&lt;img src="path" /&gt;)</option>
                  <option value="raw">Raw Path</option>
                  <option value="base64">Inline Base64 Data URI</option>
                </select>
              </div>

              <div>
                <div class="flex justify-between text-xs font-semibold text-slate-400 mb-1">
                  <span>Compression Quality</span>
                  <span class="text-orange-400">{{ config.compress_quality }}%</span>
                </div>
                <input type="range" min="10" max="100" v-model.number="config.compress_quality" class="w-full accent-orange-500 bg-slate-950" />
              </div>

              <div>
                <label class="block text-xs font-semibold text-slate-400 mb-1">Max Dimension (Pixels)</label>
                <input type="number" v-model.number="config.max_dimension" placeholder="No Limit" class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-orange-500 text-slate-200" />
              </div>
            </div>

            <!-- Right Card -->
            <div class="bg-white/[0.05] backdrop-blur-2xl border border-white/10 rounded-2xl p-6 space-y-4 shadow-[0_8px_32px_rgba(0,0,0,0.37)]">
              <h3 class="text-sm font-semibold uppercase text-slate-500 tracking-wider">System Integration</h3>

              <div>
                <label class="block text-xs font-semibold text-slate-400 mb-1">
                  Global Hotkey 
                  <span v-if="recordingHotkey" class="text-orange-400 font-bold ml-1 animate-pulse">(Recording...)</span>
                  <span v-else class="text-slate-600 normal-case font-normal ml-1">(click & press keys)</span>
                </label>
                <input type="text" readonly :value="config.global_hotkey" @focus="recordingHotkey = true" @blur="recordingHotkey = false" @keydown="recordHotkeyKeydown" :class="['w-full bg-slate-950 border rounded-xl px-3 py-2 text-sm focus:outline-none text-slate-200 font-mono cursor-pointer transition-all', recordingHotkey ? 'border-orange-500 shadow-[0_0_0_2px_rgba(249,115,22,0.2)]' : 'border-slate-800 focus:border-orange-500']" />
              </div>
              <div>
                <label class="block text-xs font-semibold text-slate-400 mb-1">
                  Screenshot Hotkey 
                  <span v-if="recordingShot" class="text-orange-400 font-bold ml-1 animate-pulse">(Recording...)</span>
                  <span v-else class="text-slate-600 normal-case font-normal ml-1">(region capture)</span>
                </label>
                <input type="text" readonly :value="config.screenshot_hotkey" @focus="recordingShot = true" @blur="recordingShot = false" @keydown="recordShotKeydown" :class="['w-full bg-slate-950 border rounded-xl px-3 py-2 text-sm focus:outline-none text-slate-200 font-mono cursor-pointer transition-all', recordingShot ? 'border-orange-500 shadow-[0_0_0_2px_rgba(249,115,22,0.2)]' : 'border-slate-800 focus:border-orange-500']" />
              </div>

              <div>
                <label class="block text-xs font-semibold text-slate-400 mb-1">Injection Mode</label>
                <select v-model="config.injection_mode" class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-orange-500 text-slate-200">
                  <option value="direct">Direct Native Keystrokes (Bypasses IME)</option>
                  <option value="swap">Quick Clipboard Swap & Paste</option>
                </select>
              </div>

              <div class="flex items-center justify-between py-1">
                <div>
                  <span class="block text-sm font-medium text-slate-200">Wrap in Single Quotes</span>
                  <span class="block text-xs text-slate-500">Wrap generated link in 'quotes'</span>
                </div>
                <label class="relative inline-flex items-center cursor-pointer">
                  <input type="checkbox" v-model="config.wrap_single_quotes" class="sr-only peer" />
                  <div class="w-11 h-6 bg-slate-800 rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-slate-300 after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-orange-500"></div>
                </label>
              </div>

              <div class="flex items-center justify-between py-1">
                <div>
                  <span class="block text-sm font-medium text-slate-200">Launch on Boot</span>
                  <span class="block text-xs text-slate-500">Start img2cli automatically</span>
                </div>
                <label class="relative inline-flex items-center cursor-pointer">
                  <input type="checkbox" v-model="config.launch_on_boot" class="sr-only peer" />
                  <div class="w-11 h-6 bg-slate-800 rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-slate-300 after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-orange-500"></div>
                </label>
              </div>

              <div class="flex items-center justify-between py-1">
                <div>
                  <span class="block text-sm font-medium text-slate-200">Enable Desktop Notifications</span>
                  <span class="block text-xs text-slate-500">Show tips on screenshot success</span>
                </div>
                <label class="relative inline-flex items-center cursor-pointer">
                  <input type="checkbox" v-model="config.enable_notifications" class="sr-only peer" />
                  <div class="w-11 h-6 bg-slate-800 rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-slate-300 after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-orange-500"></div>
                </label>
              </div>
            </div>
          </div>
          
          <!-- Save Directory Config -->
          <div class="bg-white/[0.05] backdrop-blur-2xl border border-white/10 rounded-2xl p-6 space-y-4 shadow-[0_8px_32px_rgba(0,0,0,0.37)]">
            <h3 class="text-sm font-semibold uppercase text-slate-500 tracking-wider">Advanced Paths</h3>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div>
                <label class="block text-xs font-semibold text-slate-400 mb-1">Local Temporary Directory</label>
                <input type="text" v-model="config.save_dir" placeholder="Default (Temp Dir/img2cli)" class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-orange-500 text-slate-200" />
              </div>
              <div>
                <label class="block text-xs font-semibold text-slate-400 mb-1">Clean Expired Image Files (Days)</label>
                <input type="number" v-model.number="config.clean_keep_days" class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-orange-500 text-slate-200" />
              </div>
            </div>
          </div>

          <div class="flex justify-end pt-2">
            <button @click="saveSettings" class="flex items-center gap-2 bg-gradient-to-r from-orange-500 to-amber-500 hover:from-orange-600 hover:to-amber-600 text-white px-5 py-2.5 rounded-xl font-semibold shadow-lg shadow-orange-500/20 active:scale-[0.98] transition-all duration-150 text-sm">
              Save Settings
            </button>
          </div>
        </div>

        <!-- Hosts & Targets Tab -->
        <div v-if="activeTab === 'hosts'" class="space-y-6">
          <div>
            <h2 class="text-2xl font-bold tracking-tight text-white">Hosts & Targets</h2>
            <p class="text-sm text-slate-400">Configure remote SSH servers and local workspace directory routing.</p>
          </div>

          <!-- Default SSH Config -->
          <div class="bg-white/[0.05] backdrop-blur-2xl border border-white/10 rounded-2xl p-6 space-y-4 shadow-[0_8px_32px_rgba(0,0,0,0.37)]">
            <div class="flex items-center justify-between border-b border-slate-800 pb-3">
              <h3 class="text-sm font-semibold uppercase text-slate-500 tracking-wider">Default Remote SSH Host</h3>
              <label class="relative inline-flex items-center cursor-pointer">
                <input type="checkbox" v-model="config.ssh.enabled" class="sr-only peer" />
                <div class="w-11 h-6 bg-slate-800 rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-slate-300 after:border-slate-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-orange-500"></div>
              </label>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <label class="block text-xs font-semibold text-slate-400 mb-1">Host Name</label>
                <input type="text" v-model="config.ssh.match_pattern" :disabled="!config.ssh.enabled" placeholder="e.g. My GPU Server" class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-orange-500 text-slate-200 disabled:opacity-50" />
              </div>
              <div>
                <label class="block text-xs font-semibold text-slate-400 mb-1">Host IP / Address</label>
                <input type="text" v-model="config.ssh.host" :disabled="!config.ssh.enabled" class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-orange-500 text-slate-200 disabled:opacity-50" />
              </div>
              <div>
                <label class="block text-xs font-semibold text-slate-400 mb-1">Port</label>
                <input type="number" v-model.number="config.ssh.port" :disabled="!config.ssh.enabled" class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-orange-500 text-slate-200 disabled:opacity-50" />
              </div>
              <div>
                <label class="block text-xs font-semibold text-slate-400 mb-1">Username</label>
                <input type="text" v-model="config.ssh.username" :disabled="!config.ssh.enabled" class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-orange-500 text-slate-200 disabled:opacity-50" />
              </div>
              <div class="md:col-span-2">
                <label class="block text-xs font-semibold text-slate-400 mb-1">Remote Copy Destination Folder</label>
                <input type="text" v-model="config.ssh.remote_dir" :disabled="!config.ssh.enabled" class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-orange-500 text-slate-200 disabled:opacity-50" />
              </div>
              <div>
                <label class="block text-xs font-semibold text-slate-400 mb-1">Password <span class="text-slate-600 normal-case font-normal">(OS keyring)</span></label>
                <input type="password" v-model="defaultPassword" :disabled="!config.ssh.enabled" :placeholder="defaultHasPassword ? '●●●●●● (saved) — type a new one to update' : 'blank: uses your SSH key (~/.ssh)'" class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-orange-500 text-slate-200 disabled:opacity-50" />
                <div class="flex items-center gap-2 mt-1.5">
                  <input type="checkbox" id="default-remember-pwd" v-model="config.ssh.remember_password" :disabled="!config.ssh.enabled" class="accent-orange-500 rounded bg-slate-950 border-slate-800" />
                  <label for="default-remember-pwd" class="text-xs font-medium text-slate-400 cursor-pointer select-none">Remember Password (OS Keyring)</label>
                </div>
                <div class="text-[11px] mt-1 flex items-center gap-2">
                  <template v-if="defaultHasPassword">
                    <span class="text-emerald-400">✓ Password saved (keyring)</span>
                    <button type="button" @click="clearDefaultPassword" :disabled="!config.ssh.enabled" class="text-red-400/80 hover:text-red-400 underline disabled:opacity-50">clear</button>
                  </template>
                  <span v-else class="text-slate-500">No password → will use your SSH key (~/.ssh)</span>
                </div>
              </div>
            </div>

            <div class="flex justify-end pt-2">
              <button 
                @click="checkSSHConnection" 
                :disabled="!config.ssh.enabled || testingConnection"
                class="bg-slate-800 hover:bg-slate-700 text-slate-200 font-semibold px-4 py-2 rounded-xl text-xs active:scale-[0.98] transition-all disabled:opacity-50 flex items-center gap-2"
              >
                <span v-if="testingConnection" class="w-3 h-3 border-2 border-slate-400 border-t-transparent rounded-full animate-spin"></span>
                {{ testingConnection ? 'Testing...' : 'Test Connection' }}
              </button>
            </div>
          </div>

          <!-- Dynamic Targets List -->
          <div class="bg-white/[0.05] backdrop-blur-2xl border border-white/10 rounded-2xl p-6 space-y-4 shadow-[0_8px_32px_rgba(0,0,0,0.37)]">
            <div class="flex items-center justify-between border-b border-slate-800 pb-3">
              <h3 class="text-sm font-semibold uppercase text-slate-500 tracking-wider">Dynamic Router Targets</h3>
              <div class="flex items-center gap-2">
                <button
                  @click="openSshLoader"
                  :disabled="loadingSsh"
                  class="bg-white/5 hover:bg-white/10 border border-white/10 text-slate-200 font-semibold px-3 py-1.5 rounded-xl text-xs flex items-center gap-1 active:scale-[0.98] transition-all disabled:opacity-50"
                >
                  <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
                  </svg>
                  Load SSH Config
                </button>
                <button
                  @click="showAddTargetModal = true"
                  class="bg-orange-500 hover:bg-orange-600 text-white font-semibold px-3 py-1.5 rounded-xl text-xs flex items-center gap-1 active:scale-[0.98] transition-all"
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
                  <tr class="border-b border-slate-850 text-xs font-semibold text-slate-500">
                    <th class="w-16 py-3 px-4 text-center">Status</th>
                    <th class="w-40 py-3 px-4 text-left">Host Name / Alias</th>
                    <th class="w-24 py-3 px-4 text-center">Type</th>
                    <th class="py-3 px-4 text-left">Details</th>
                    <th class="w-56 py-3 px-4 text-center">Actions</th>
                  </tr>
                </thead>
                <tbody class="divide-y divide-slate-800/40 text-sm">
                  <tr v-for="(target, idx) in (config.targets || [])" :key="idx" class="hover:bg-slate-900/20">
                    <td class="py-3 px-4 text-center">
                      <label class="relative inline-flex items-center cursor-pointer">
                        <input type="checkbox" v-model="target.enabled" class="sr-only peer" />
                        <div class="w-9 h-5 bg-slate-800 rounded-full peer peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-slate-300 after:border-slate-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-orange-500"></div>
                      </label>
                    </td>
                    <td class="py-3 px-4 font-semibold text-slate-200 max-w-[10rem] truncate">{{ target.match_pattern }}</td>
                    <td class="py-3 px-4 text-center">
                      <span :class="['px-2 py-0.5 rounded-md text-xs font-semibold uppercase', target.type === 'ssh' ? 'bg-orange-500/10 text-orange-400 border border-orange-500/25' : 'bg-amber-500/10 text-amber-400 border border-amber-500/25']">
                        {{ target.type }}
                      </span>
                    </td>
                    <td class="py-3 px-4 text-xs text-slate-400 max-w-[28rem] truncate">
                      <span v-if="target.type === 'ssh'" class="block truncate" :title="`${target.username}@${target.host}:${target.remote_dir}`">{{ target.username }}@{{ target.host }}:{{ target.remote_dir }}</span>
                      <span v-else class="block truncate" :title="target.local_dir">{{ target.local_dir }}</span>
                    </td>
                    <td class="py-3 px-4 text-center">
                      <div class="flex items-center justify-center gap-1.5">
                        <button v-if="target.type === 'ssh'" @click="setAsDefault(idx)" class="px-1.5 py-0.5 rounded-md text-[11px] font-semibold bg-orange-500/10 text-orange-400 border border-orange-500/25 hover:bg-orange-500/20 transition-colors">Set Default</button>
                        <button @click="editTarget(idx)" class="px-1.5 py-0.5 rounded-md text-[11px] font-semibold bg-slate-400/10 text-slate-300 border border-slate-400/25 hover:bg-slate-400/20 transition-colors">Edit</button>
                        <button @click="deleteTarget(idx)" class="px-1.5 py-0.5 rounded-md text-[11px] font-semibold bg-red-500/10 text-red-400 border border-red-500/25 hover:bg-red-500/20 transition-colors">Delete</button>
                      </div>
                    </td>
                  </tr>
                  <tr v-if="!(config.targets || []).length">
                    <td colspan="5" class="py-6 text-center text-slate-500 text-xs">No routing targets configured. Clipboard uploads will fallback to default host.</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>

          <div class="flex justify-end pt-2">
            <button @click="saveSettings" class="flex items-center gap-2 bg-gradient-to-r from-orange-500 to-amber-500 hover:from-orange-600 hover:to-amber-600 text-white px-5 py-2.5 rounded-xl font-semibold shadow-lg shadow-orange-500/20 active:scale-[0.98] transition-all duration-150 text-sm">
              Save Settings
            </button>
          </div>
        </div>

        <!-- System Logs Tab -->
        <div v-if="activeTab === 'logs'" class="space-y-6 flex flex-col h-[calc(100vh-8rem)]">
          <div class="flex justify-between items-center shrink-0">
            <div>
              <h2 class="text-2xl font-bold tracking-tight text-white">System Logs</h2>
              <p class="text-sm text-slate-400">Real-time daemon events and screenshot processing logs.</p>
            </div>
            <button @click="logs = []" class="bg-slate-800 hover:bg-slate-700 text-slate-200 font-semibold px-3 py-1.5 rounded-xl text-xs active:scale-[0.98] transition-all">
              Clear Logs
            </button>
          </div>

          <div class="flex-1 bg-slate-950 border border-slate-850 rounded-2xl p-4 overflow-y-auto font-mono text-xs text-slate-400 space-y-1.5 shadow-inner" ref="logContainer">
            <div v-for="(log, idx) in logs" :key="idx" class="whitespace-pre-wrap leading-relaxed">
              <span class="text-slate-600 select-none">[{{ idx + 1 }}]</span> {{ log }}
            </div>
            <div v-if="!logs.length" class="text-slate-600 text-center py-12">No logs loaded. Press global hotkey to trigger daemon activity.</div>
          </div>
        </div>

      </main>
    </div>

    <!-- Add/Edit Target Modal -->
    <div v-if="showAddTargetModal" class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50">
      <div class="bg-white/[0.07] backdrop-blur-2xl border border-white/10 rounded-2xl max-w-lg w-full p-6 space-y-4 shadow-2xl">
        <h3 class="text-lg font-bold text-white">{{ editingTargetIndex !== null ? 'Edit Router Target' : 'Add Router Target' }}</h3>
        
        <div class="space-y-3">
          <div class="grid grid-cols-2 gap-3">
            <div>
              <label class="block text-xs font-semibold text-slate-400 mb-1">Target Type</label>
              <select v-model="tempTarget.type" class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-orange-500 text-slate-200">
                <option value="ssh">SSH (Remote Server)</option>
                <option value="local">Local Folder</option>
              </select>
            </div>
            <div>
              <label class="block text-xs font-semibold text-slate-400 mb-1">Host Name / Alias</label>
              <input type="text" v-model="tempTarget.match_pattern" placeholder="e.g. GPU-90, WSL" class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-orange-500 text-slate-200" />
            </div>
          </div>

          <!-- SSH Target Fields -->
          <div v-if="tempTarget.type === 'ssh'" class="grid grid-cols-2 gap-3">
            <div>
              <label class="block text-xs font-semibold text-slate-400 mb-1">Host IP / Address</label>
              <input type="text" v-model="tempTarget.host" class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-orange-500 text-slate-200" />
            </div>
            <div>
              <label class="block text-xs font-semibold text-slate-400 mb-1">Port</label>
              <input type="number" v-model.number="tempTarget.port" class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-orange-500 text-slate-200" />
            </div>
            <div>
              <label class="block text-xs font-semibold text-slate-400 mb-1">Username</label>
              <input type="text" v-model="tempTarget.username" class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-orange-500 text-slate-200" />
            </div>
            <div>
              <label class="block text-xs font-semibold text-slate-400 mb-1">Remote Copy Destination Folder</label>
              <input type="text" v-model="tempTarget.remote_dir" class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-orange-500 text-slate-200" />
            </div>
            <div>
              <label class="block text-xs font-semibold text-slate-400 mb-1">Password <span class="text-slate-600 normal-case font-normal">(OS keyring)</span></label>
              <input type="password" v-model="tempTarget.password" :placeholder="tempTargetHasPassword ? '●●●●●● (saved) — type a new one to update' : 'blank: uses your SSH key (~/.ssh)'" class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-orange-500 text-slate-200" />
              <div class="flex items-center gap-2 mt-1.5">
                <input type="checkbox" id="target-remember-pwd" v-model="tempTarget.remember_password" class="accent-orange-500 rounded bg-slate-950 border-slate-800" />
                <label for="target-remember-pwd" class="text-xs font-medium text-slate-400 cursor-pointer select-none">Remember Password (OS Keyring)</label>
              </div>
              <div class="text-[11px] mt-1 flex items-center gap-2">
                <template v-if="tempTargetHasPassword">
                  <span class="text-emerald-400">✓ Password saved (keyring)</span>
                  <button type="button" @click="clearTargetPassword" class="text-red-400/80 hover:text-red-400 underline">clear</button>
                </template>
                <span v-else class="text-slate-500">No password → will use your SSH key (~/.ssh)</span>
              </div>
            </div>
          </div>

          <!-- Local Target Fields -->
          <div v-if="tempTarget.type === 'local'">
            <label class="block text-xs font-semibold text-slate-400 mb-1">Local Copy Destination Folder</label>
            <input type="text" v-model="tempTarget.local_dir" placeholder="e.g. C:\users\docs\images" class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-orange-500 text-slate-200" />
          </div>
        </div>

        <div class="flex justify-end gap-3 pt-3 border-t border-slate-800">
          <button @click="closeTargetModal" class="bg-slate-800 hover:bg-slate-700 text-slate-200 px-4 py-2 rounded-xl text-xs font-semibold">Cancel</button>
          <button @click="saveTarget" class="bg-orange-500 hover:bg-orange-600 text-white px-4 py-2 rounded-xl text-xs font-semibold">Save</button>
        </div>
      </div>
    </div>

    <!-- SSH Config Loader Modal -->
    <div v-if="showSshModal" class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center p-4 z-50">
      <div class="bg-white/[0.07] backdrop-blur-2xl border border-white/10 rounded-2xl max-w-lg w-full p-6 space-y-4 shadow-2xl">
        <h3 class="text-lg font-bold text-white">Load OpenSSH config</h3>
        <div class="flex items-center gap-2">
          <input type="text" v-model="sshConfigPath" placeholder="~/.ssh/config" class="flex-1 bg-slate-950/60 border border-white/10 rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-orange-500 text-slate-200 font-mono" />
          <button @click="browseSshConfig" :disabled="loadingSsh" class="bg-white/5 hover:bg-white/10 border border-white/10 text-slate-200 font-semibold px-3 py-2 rounded-xl text-xs disabled:opacity-50 whitespace-nowrap">Browse…</button>
          <button @click="openSshLoader" :disabled="loadingSsh" class="bg-white/5 hover:bg-white/10 border border-white/10 text-slate-200 font-semibold px-3 py-2 rounded-xl text-xs disabled:opacity-50 whitespace-nowrap">Load</button>
        </div>
        <input type="text" v-model="sshSearch" placeholder="Search hosts (alias / host / user)..." class="w-full bg-slate-950/60 border border-white/10 rounded-xl px-3 py-2 text-sm focus:outline-none focus:border-orange-500 text-slate-200" />
        <div class="max-h-72 overflow-y-auto space-y-1 pr-1">
          <label v-for="{ h, i } in filteredSshHosts" :key="i" class="flex items-center gap-3 px-3 py-2 rounded-lg hover:bg-white/5 cursor-pointer">
            <input type="checkbox" v-model="sshSelected[i]" class="accent-orange-500 w-4 h-4" />
            <div class="flex-1 min-w-0">
              <div class="text-sm font-semibold text-slate-100 truncate">{{ h.alias }}</div>
              <div class="text-xs text-slate-400 truncate font-mono">{{ h.username }}@{{ h.host }}:{{ h.port }}</div>
            </div>
          </label>
          <div v-if="!filteredSshHosts.length" class="text-slate-500 text-center py-8 text-sm">No hosts found.</div>
        </div>
        <div class="flex items-center justify-between pt-3 border-t border-white/10">
          <button @click="toggleAllSsh(true)" class="text-xs font-semibold text-slate-300 hover:text-white transition-colors">Select All</button>
          <div class="flex gap-3">
            <button @click="closeSshModal" class="bg-white/5 hover:bg-white/10 text-slate-200 px-4 py-2 rounded-xl text-xs font-semibold">Cancel</button>
            <button @click="importSshSelected" class="bg-orange-500 hover:bg-orange-600 text-white px-4 py-2 rounded-xl text-xs font-semibold">Import Selected</button>
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
import { ref, onMounted, nextTick, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { open as openDialog } from '@tauri-apps/plugin-dialog';

// Active Tab
const activeTab = ref('general');

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
const setAsDefault = (index) => {
  const t = config.value.targets[index];
  if (!t || t.type !== 'ssh') return;
  config.value.ssh = {
    enabled: true,
    host: t.host || '',
    port: t.port || 22,
    username: t.username || '',
    remote_dir: t.remote_dir || config.value.ssh?.remote_dir || '/tmp/img2cli',
    match_pattern: config.value.ssh?.match_pattern || ''
  };
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
    return;
  }
  loadConfig();
  setupLogs();
});
</script>

<style>
/* Custom styled range slider */
input[type="range"]::-webkit-slider-thumb {
  height: 16px;
  width: 16px;
  border-radius: 50%;
  background: #f97316;
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
  border: 1px border #1e293b;
}
</style>
