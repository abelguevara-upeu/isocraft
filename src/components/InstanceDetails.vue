<template>
  <div class="flex-1 flex flex-col h-full overflow-hidden bg-[var(--bg-main)] animate-fade-in">
    <!-- Banner / Header -->
    <header class="relative h-32 flex-shrink-0 overflow-hidden border-b border-[var(--border-color)]">
      <!-- Dynamic Background Gradient -->
      <div class="absolute inset-0 bg-gradient-to-br from-green-900/10 via-[#0a0d12] to-[#0a0d12]"></div>
      
      <!-- Content Overlay -->
      <div class="relative z-10 h-full flex flex-col justify-end p-5">
        <div class="flex justify-between items-end w-full">
          <div>
            <h2 class="text-2xl font-black text-white tracking-tighter drop-shadow-2xl mb-2">{{ instance.name }}</h2>
            <div class="flex items-center gap-2">
              <div class="flex items-center gap-1 px-2 py-0.5 bg-white/5 border border-white/10 rounded-md backdrop-blur-sm shadow-inner">
                <span class="text-[8px] font-black uppercase tracking-widest text-gray-500">Loader</span>
                <span class="text-[10px] font-bold text-green-500">{{ instance.loader }} {{ instance.loader_version !== instance.version_id ? instance.loader_version : '' }}</span>
              </div>
              <div class="flex items-center gap-1 px-2 py-0.5 bg-white/5 border border-white/10 rounded-md backdrop-blur-sm shadow-inner">
                <span class="text-[8px] font-black uppercase tracking-widest text-gray-400">Version</span>
                <span class="text-[10px] font-bold text-blue-400 font-mono">{{ instance.version_id }}</span>
              </div>
              <div class="flex items-center gap-1 px-2 py-0.5 bg-white/5 border border-white/10 rounded-md backdrop-blur-sm shadow-inner">
                <span class="text-[8px] font-black uppercase tracking-widest text-gray-500">RAM</span>
                <span class="text-[10px] font-bold text-purple-400">{{ instance.max_memory }}</span>
              </div>
            </div>
          </div>

          <div class="flex gap-1.5 mb-0.5 items-center">
            <!-- Relocated Play Button -->
            <button 
              @click="handleLaunch"
              :disabled="launchState.isLaunching"
              class="px-4 py-2 rounded-lg text-xs font-black tracking-wider flex items-center gap-1.5 transition-all group shadow-md"
              :class="launchState.isPlaying 
                ? 'bg-gray-800 text-green-500 border border-green-500/20' 
                : 'bg-green-600 hover:bg-green-500 text-white active:scale-95 shadow-green-950/20'"
            >
              <template v-if="launchState.isPlaying">
                 <div class="w-1.5 h-1.5 bg-green-500 rounded-full animate-ping"></div>
                 RUNNING
              </template>
              <template v-else-if="launchState.isLaunching">
                 <svg class="w-3 h-3 animate-spin" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" /></svg>
                 PREPARING...
              </template>
              <template v-else>
                 <svg class="w-3 h-3 transition-transform group-hover:scale-110" fill="currentColor" viewBox="0 0 24 24"><path d="M8 5v14l11-7z"/></svg>
                 PLAY
              </template>
            </button>

            <!-- Line Divider -->
            <div class="h-5 w-[1px] bg-white/10 mx-1"></div>

            <button @click="$emit('edit')" class="btn-secondary group hover:border-purple-500/50 hover:bg-purple-500/10 text-purple-500/80 hover:text-purple-400 px-3 py-2 text-xs transition-all" title="Edit Settings">
              <svg class="w-3.5 h-3.5 group-hover:rotate-90 transition-transform duration-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
              </svg>
            </button>
            <button @click="$emit('open-folder')" class="btn-secondary group px-3 py-2 text-xs" title="Open Folder">
              <svg class="w-3.5 h-3.5 group-hover:scale-110 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
              </svg>
            </button>
            <button @click="$emit('delete')" class="btn-secondary group border-red-900/30 hover:border-red-500/50 hover:bg-red-500/10 text-red-500/80 hover:text-red-500 px-3 py-2 text-xs transition-all" title="Delete Instance">
              <svg class="w-3.5 h-3.5 group-hover:rotate-12 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
              </svg>
            </button>
          </div>
        </div>
      </div>
    </header>

    <!-- Navigation Tabs -->
    <nav class="flex gap-5 px-5 pt-2 border-b border-[var(--border-color)] bg-black/20 flex-shrink-0">
      <button 
        v-for="tab in ['general', 'mods', 'resourcepacks']" 
        :key="tab"
        @click="activeTab = tab"
        :class="[
          'pb-2 text-[9px] font-black uppercase tracking-[0.2em] transition-all relative',
          activeTab === tab ? 'text-white' : 'text-gray-500 hover:text-gray-300'
        ]"
      >
        {{ tab }}
        <div v-if="activeTab === tab" class="absolute bottom-0 left-0 right-0 h-0.5 bg-green-500 rounded-t-full shadow-[0_0_10px_rgba(34,197,94,0.5)]"></div>
      </button>
    </nav>

    <!-- Status Ticker Ticker Bar (Only visible when launching) -->
    <div v-if="launchState.isLaunching" class="w-full bg-black/40 px-5 py-1 border-b border-white/5 flex items-center justify-between text-[9px] font-mono uppercase tracking-widest text-green-500 flex-shrink-0">
      <span class="animate-pulse">{{ launchState.message }}</span>
      <span class="text-white font-bold">{{ launchState.progress }}%</span>
    </div>

    <!-- Status Slim Progress Line (Only visible when launching) -->
    <div v-if="launchState.isLaunching" class="w-full bg-gray-950 h-0.5 overflow-hidden flex-shrink-0">
      <div 
        class="bg-gradient-to-r from-green-600 via-green-400 to-green-500 h-full transition-all duration-300 relative shadow-[0_0_10px_rgba(34,197,94,0.5)]"
        :style="{ width: launchState.progress + '%' }"
      >
        <div class="absolute inset-0 bg-white/20 animate-pulse"></div>
      </div>
    </div>

    <!-- Content Area (Solid dark background, padding moved to children to fill the screen edge-to-edge) -->
    <div class="flex-1 overflow-hidden flex flex-col min-h-0 bg-black/35 border-l border-white/5">
      
      <!-- Tab Content: General -->
      <div v-if="activeTab === 'general'" class="flex-1 overflow-hidden p-5 flex flex-col min-h-0">
        
        <!-- Dashboard Grid (2 columns: Left for configuration/profile, Right for scrollable media lists) -->
        <div class="grid grid-cols-[300px,1fr] gap-4 flex-1 min-h-0 overflow-hidden">
          
          <!-- LEFT COLUMN: Profile & Settings -->
          <div class="flex flex-col gap-4 overflow-y-auto custom-scrollbar pr-1">
            
            <!-- Instance Profile Card -->
            <div class="premium-card p-4 bg-gradient-to-br from-gray-800/10 to-transparent border border-white/5 flex flex-col gap-3">
              <h3 class="text-[9px] font-black uppercase tracking-[0.2em] text-gray-500 flex items-center gap-1.5">
                <svg class="w-3 h-3 text-blue-500" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
                Instance Profile
              </h3>
              
              <!-- Compact Information -->
              <div class="flex flex-col gap-2 text-[10px] font-sans text-gray-400">
                <div class="flex justify-between border-b border-white/5 pb-1.5">
                  <span class="text-gray-500 font-medium">Last Played</span>
                  <span class="text-white font-bold">{{ instance.last_played ? formatTime(instance.last_played) : 'Never' }}</span>
                </div>
                <div class="flex justify-between border-b border-white/5 pb-1.5">
                  <span class="text-gray-500 font-medium">Created On</span>
                  <span class="text-white font-bold">{{ formatDate(instance.created_at) }}</span>
                </div>
                <div class="flex flex-col gap-1 mt-1">
                  <span class="text-gray-500 font-medium">Path Directory</span>
                  <div class="bg-black/40 rounded px-2 py-1 border border-white/5 font-mono text-[9px] text-gray-400 break-all select-all flex items-center justify-between">
                    <span>~/.isocraft/instances/{{ instance.name }}</span>
                  </div>
                </div>
              </div>
            </div>

            <!-- Game Launcher Settings Card (Read-only Summary!) -->
            <div class="premium-card p-4 bg-gradient-to-br from-gray-800/10 to-transparent border border-white/5 flex flex-col gap-3">
              <h3 class="text-[9px] font-black uppercase tracking-[0.2em] text-gray-500 flex items-center gap-1.5">
                <svg class="w-3 h-3 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" /></svg>
                Runtime Config
              </h3>
              
              <div class="flex flex-col gap-2.5 text-[10px] font-sans text-gray-400">
                <div class="flex items-center justify-between border-b border-white/5 pb-1.5">
                  <span class="text-gray-500 font-medium flex items-center gap-1.5">
                    <svg class="w-3.5 h-3.5 text-gray-500 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" /></svg>
                    Username
                  </span>
                  <span class="text-white font-bold">{{ instance.username || 'Player' }}</span>
                </div>
                
                <div class="flex items-center justify-between border-b border-white/5 pb-1.5">
                  <span class="text-gray-500 font-medium flex items-center gap-1.5">
                    <svg class="w-3.5 h-3.5 text-gray-500 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" /></svg>
                    Allocated RAM
                  </span>
                  <span class="text-green-400 font-bold font-mono">{{ instance.max_memory ? instance.max_memory.replace('G', ' GB') : '4 GB' }}</span>
                </div>

                <div class="flex items-center justify-between border-b border-white/5 pb-1.5">
                  <span class="text-gray-500 font-medium flex items-center gap-1.5">
                    <svg class="w-3.5 h-3.5 text-gray-500 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 3v4M3 5h4M6 17v4m-2-2h4m5-16l2.286 6.857L21 12l-5.714 2.143L13 21l-2.286-6.857L5 12l5.714-2.143L13 3z" /></svg>
                    Mod Loader
                  </span>
                  <span class="text-purple-400 font-bold">{{ instance.loader }}</span>
                </div>

                <div v-if="instance.loader !== 'Vanilla'" class="flex items-center justify-between">
                  <span class="text-gray-500 font-medium flex items-center gap-1.5">
                    <svg class="w-3.5 h-3.5 text-gray-500 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 7h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" /></svg>
                    Loader Version
                  </span>
                  <span class="text-white font-mono font-bold text-[9px] bg-white/5 border border-white/10 px-1.5 py-0.5 rounded leading-none">
                    {{ instance.loader_version }}
                  </span>
                </div>
              </div>
            </div>

            <!-- Quick Open Folders Panel (Sleek Icon Row, no text buttons!) -->
            <div class="premium-card p-4 bg-gradient-to-br from-gray-800/10 to-transparent border border-white/5 flex flex-col gap-2.5">
              <h3 class="text-[9px] font-black uppercase tracking-[0.2em] text-gray-500 flex items-center gap-1.5">
                <svg class="w-3 h-3 text-purple-500" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" /></svg>
                Explore Folders
              </h3>
              <div class="flex justify-between gap-1 mt-0.5">
                <button @click="handleOpenSubPath('saves')" class="flex-1 p-2 bg-black/40 border border-white/5 hover:border-amber-500/30 hover:bg-amber-500/5 rounded-lg flex flex-col items-center gap-1 group transition-all" title="Open Worlds (saves) Folder">
                  <svg class="w-4 h-4 text-amber-500/70 group-hover:text-amber-500" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 002 2h2.945M11 20.955V18.5a2.5 2.5 0 012.5-2.5h0a2.5 2.5 0 012.5 2.5v2.455M21 12a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
                  <span class="text-[8px] font-bold text-gray-500 group-hover:text-gray-300">saves</span>
                </button>
                <button @click="handleOpenSubPath('screenshots')" class="flex-1 p-2 bg-black/40 border border-white/5 hover:border-pink-500/30 hover:bg-pink-500/5 rounded-lg flex flex-col items-center gap-1 group transition-all" title="Open Screenshots Folder">
                  <svg class="w-4 h-4 text-pink-500/70 group-hover:text-pink-500" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" /></svg>
                  <span class="text-[8px] font-bold text-gray-500 group-hover:text-gray-300">screens</span>
                </button>
                <button @click="handleOpenLogs" class="flex-1 p-2 bg-black/40 border border-white/5 hover:border-green-500/30 hover:bg-green-500/5 rounded-lg flex flex-col items-center gap-1 group transition-all" title="Open Game Logs (latest.log)">
                  <svg class="w-4 h-4 text-green-500/70 group-hover:text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" /></svg>
                  <span class="text-[8px] font-bold text-gray-500 group-hover:text-gray-300">logs</span>
                </button>
              </div>
            </div>

          </div>

          <!-- RIGHT COLUMN: Saves & Screenshots split vertically -->
          <div class="flex flex-col gap-4 min-h-0 overflow-hidden">
            
            <!-- Saves (Worlds) list -->
            <div class="flex-1 flex flex-col min-h-0 premium-card p-4 bg-gradient-to-br from-gray-800/10 to-transparent border border-white/5">
              <h3 class="text-[9px] font-black uppercase tracking-[0.2em] text-gray-500 mb-3 flex items-center justify-between flex-shrink-0">
                <span class="flex items-center gap-1.5">
                  <svg class="w-3 h-3 text-amber-500" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 002 2h2.945M11 20.955V18.5a2.5 2.5 0 012.5-2.5h0a2.5 2.5 0 012.5 2.5v2.455M21 12a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
                  Singleplayer Worlds (Saves)
                </span>
                <span class="text-[8px] font-mono font-bold text-gray-500 bg-white/5 border border-white/10 px-1.5 py-0.5 rounded leading-none">
                  {{ saves.length }} worlds
                </span>
              </h3>

              <div v-if="saves.length > 0" class="flex-1 overflow-y-auto custom-scrollbar pr-1 flex flex-col gap-2 content-start">
                <div 
                  v-for="world in saves" 
                  :key="world" 
                  class="bg-black/30 border border-white/5 rounded-md flex flex-col hover:border-amber-500/20 hover:bg-white/[0.02] group transition-all"
                >
                  <!-- World Header -->
                  <div class="py-2 px-3 flex items-center justify-between">
                    <div class="flex items-center gap-2.5 min-w-0 flex-1">
                      <svg class="w-3.5 h-3.5 text-amber-500/80 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z" /><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 11a3 3 0 11-6 0 3 3 0 016 0z" /></svg>
                      <span class="text-[10px] font-bold text-white truncate" :title="world">{{ world }}</span>
                    </div>
                    
                    <div class="flex items-center gap-1.5 shrink-0 ml-2">
                      <!-- Datapacks Toggle Button -->
                      <button 
                        @click="toggleDatapacks(world)"
                        class="px-2 py-0.5 text-[8px] font-black uppercase tracking-wider text-amber-500/80 hover:text-amber-400 bg-amber-500/10 hover:bg-amber-500/20 border border-amber-500/20 rounded transition-all flex items-center gap-1"
                        :class="{ 'bg-amber-500/30 text-amber-300 border-amber-500/40': expandedWorldDatapacks === world }"
                        title="Manage Datapacks"
                      >
                        <svg class="w-2.5 h-2.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" /></svg>
                        datapacks
                      </button>
                      
                      <!-- Open Folder Button -->
                      <button 
                        @click="handleOpenSubPath('saves/' + world)"
                        class="p-1 text-gray-500 hover:text-white rounded hover:bg-white/10 transition-all"
                        title="Open World Folder"
                      >
                        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" /></svg>
                      </button>
                      
                      <!-- Delete Button -->
                      <button 
                        @click="handleDeleteSave(world)"
                        class="p-1 text-red-500/40 hover:text-red-500 rounded hover:bg-red-500/10 transition-all"
                        title="Delete World"
                      >
                        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" /></svg>
                      </button>
                    </div>
                  </div>
                  
                  <!-- Datapacks Inline Panel -->
                  <div 
                    v-if="expandedWorldDatapacks === world" 
                    class="border-t border-white/5 bg-black/45 p-3 rounded-b-md flex flex-col gap-2.5"
                  >
                    <div class="flex items-center justify-between">
                      <span class="text-[8px] font-black uppercase tracking-wider text-gray-500 flex items-center gap-1">
                        <svg class="w-2.5 h-2.5 text-amber-500" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" /></svg>
                        World Datapacks
                      </span>
                      
                      <!-- Import Actions -->
                      <div class="flex items-center gap-1 shrink-0">
                        <button 
                          @click="handleImportDatapack(world, 'file')"
                          class="px-1.5 py-0.5 text-[8px] font-bold text-gray-400 hover:text-white bg-white/5 border border-white/10 rounded transition-all"
                        >
                          + Zip File
                        </button>
                        <button 
                          @click="handleImportDatapack(world, 'directory')"
                          class="px-1.5 py-0.5 text-[8px] font-bold text-gray-400 hover:text-white bg-white/5 border border-white/10 rounded transition-all"
                        >
                          + Folder
                        </button>
                      </div>
                    </div>
                    
                    <!-- Datapacks List -->
                    <div v-if="datapacksMap[world] && datapacksMap[world].length > 0" class="flex flex-col gap-1 max-h-[140px] overflow-y-auto custom-scrollbar pr-1">
                      <div 
                        v-for="dp in datapacksMap[world]" 
                        :key="dp" 
                        class="bg-white/[0.02] border border-white/5 py-1 px-2.5 rounded flex items-center justify-between"
                      >
                        <span class="text-[9px] font-mono text-gray-300 truncate pr-2 flex items-center gap-1.5" :title="dp">
                          <svg class="w-2.5 h-2.5 text-amber-500/60" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" /></svg>
                          {{ dp }}
                        </span>
                        
                        <button 
                          @click="handleDeleteDatapack(world, dp)"
                          class="p-0.5 text-red-500/40 hover:text-red-500 rounded hover:bg-red-500/10 transition-all shrink-0"
                          title="Delete Datapack"
                        >
                          <svg class="w-2.5 h-2.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" /></svg>
                        </button>
                      </div>
                    </div>
                    
                    <div v-else class="text-center py-3 border border-dashed border-white/5 rounded-md bg-white/[0.01]">
                      <span class="text-[8px] text-gray-500 font-sans">No datapacks active. Click + to add one.</span>
                    </div>
                  </div>
                </div>
              </div>
              <div v-else class="flex-1 flex flex-col items-center justify-center border border-dashed border-white/5 rounded-lg bg-black/10 p-5">
                <svg class="w-6 h-6 text-gray-700 mb-1.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M18.364 5.636l-3.536 3.536m0 5.656l3.536 3.536M9.172 9.172L5.636 5.636m3.536 9.192l-3.536 3.536M21 12a9 9 0 11-18 0 9 9 0 0118 0zm-5 0a4 4 0 11-8 0 4 4 0 018 0z" /></svg>
                <p class="text-[10px] font-bold text-gray-500">No worlds found</p>
                <p class="text-[9px] text-gray-600 font-sans mt-0.5 text-center">Start Minecraft and create a new world to see it here</p>
              </div>
            </div>

            <!-- Screenshots list -->
            <div class="flex-1 flex flex-col min-h-0 premium-card p-4 bg-gradient-to-br from-gray-800/10 to-transparent border border-white/5">
              <h3 class="text-[9px] font-black uppercase tracking-[0.2em] text-gray-500 mb-3 flex items-center justify-between flex-shrink-0">
                <span class="flex items-center gap-1.5">
                  <svg class="w-3 h-3 text-pink-500" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" /></svg>
                  Screenshots Browser
                </span>
                <span class="text-[8px] font-mono font-bold text-gray-500 bg-white/5 border border-white/10 px-1.5 py-0.5 rounded leading-none">
                  {{ screenshots.length }} images
                </span>
              </h3>

              <div v-if="screenshots.length > 0" class="flex-1 overflow-y-auto custom-scrollbar pr-1 grid grid-cols-[repeat(auto-fill,minmax(200px,1fr))] gap-2 content-start">
                <div 
                  v-for="img in screenshots" 
                  :key="img" 
                  class="bg-black/30 border border-white/5 py-1.5 px-3 rounded-md flex items-center justify-between hover:border-pink-500/30 hover:bg-white/5 group transition-all"
                >
                  <div class="flex items-center gap-2.5 min-w-0 flex-1">
                    <svg class="w-3.5 h-3.5 text-pink-500/80 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" /></svg>
                    <div class="flex flex-col min-w-0 flex-1">
                      <span class="text-[10px] font-bold text-white truncate font-sans" :title="img">{{ formatScreenshotName(img) }}</span>
                      <span v-if="getScreenshotTimeIdentifier(img)" class="text-[8px] font-mono text-gray-500/80 truncate mt-0.5">{{ getScreenshotTimeIdentifier(img) }}</span>
                    </div>
                  </div>
                  <div class="flex items-center gap-1 shrink-0 ml-2">
                    <button 
                      @click="handleOpenSubPath('screenshots/' + img)"
                      class="p-1 text-gray-500 hover:text-white rounded hover:bg-white/10 transition-all"
                      title="View Image"
                    >
                      <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" /><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" /></svg>
                    </button>
                    <button 
                      @click="handleDeleteScreenshot(img)"
                      class="p-1 text-red-500/40 hover:text-red-500 rounded hover:bg-red-500/10 transition-all"
                      title="Delete Screenshot"
                    >
                      <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" /></svg>
                    </button>
                  </div>
                </div>
              </div>
              <div v-else class="flex-1 flex flex-col items-center justify-center border border-dashed border-white/5 rounded-lg bg-black/10 p-5">
                <svg class="w-6 h-6 text-gray-700 mb-1.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" /></svg>
                <p class="text-[10px] font-bold text-gray-500">No screenshots found</p>
                <p class="text-[9px] text-gray-600 font-sans mt-0.5 text-center">Press F2 inside Minecraft to take screenshots</p>
              </div>
            </div>

          </div>

        </div>

      </div>

      <!-- Tab Content: Files -->
      <div v-else class="flex-1 flex flex-col min-h-0 relative">
        <!-- Search and Filter bar -->
        <div v-if="files.length > 0" class="px-5 pt-4 pb-2 flex items-center justify-between gap-4 flex-shrink-0">
          <div class="relative flex-1 max-w-sm">
            <span class="absolute inset-y-0 left-0 pl-2.5 flex items-center pointer-events-none text-gray-500">
              <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" /></svg>
            </span>
            <input 
              v-model="searchQuery" 
              type="text" 
              placeholder="Search files..." 
              class="input-field pl-8 py-1 text-xs w-full bg-black/40 border-white/5 focus:border-purple-500/50 rounded-md" 
            />
          </div>
          <span class="text-[9px] font-bold text-gray-500 font-mono tracking-wider shrink-0 bg-white/5 border border-white/10 px-2 py-0.5 rounded shadow-inner">
            <template v-if="searchQuery.trim()">
              {{ filteredFiles.length }} / {{ files.length }} items
            </template>
            <template v-else>
              {{ files.length }} items
            </template>
          </span>
        </div>

        <!-- Scrollable Grid of items (borderless) -->
        <div v-if="filteredFiles.length > 0" class="px-5 pb-5 grid grid-cols-[repeat(auto-fill,minmax(280px,1fr))] gap-1.5 overflow-y-auto custom-scrollbar flex-1 content-start pr-3">
          <div 
            v-for="file in filteredFiles" 
            :key="file" 
            class="bg-black/20 border border-white/5 py-1.5 px-3 rounded-md flex items-center justify-between hover:border-gray-700/60 hover:bg-white/5 group shadow-sm cursor-pointer w-full"
          >
             <div class="flex items-center gap-2 min-w-0 flex-1 justify-between pr-2">
                <div class="flex items-center gap-2 min-w-0 flex-1">
                   <svg class="w-3.5 h-3.5 text-blue-500/80 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z" /></svg>
                   <span class="text-[10px] font-black text-white truncate" :title="file">{{ formatFileName(file).title }}</span>
                </div>
                <span v-if="formatFileName(file).version" class="text-[8px] font-mono font-bold text-gray-500 shrink-0 bg-white/[0.02] border border-white/5 px-1 py-0.5 rounded leading-none" :title="formatFileName(file).version">
                   {{ formatFileName(file).version }}
                </span>
             </div>
             <button 
               @click.stop="$emit('delete-file', file)" 
               class="p-1 text-red-500/40 hover:text-red-500 rounded hover:bg-red-500/10 shrink-0 ml-1.5 transition-all"
               title="Delete File"
             >
               <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                 <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
               </svg>
             </button>
          </div>
        </div>
        
        <!-- Empty State (also borderless) -->
        <div v-else class="flex-1 flex flex-col items-center justify-center p-12 text-center opacity-50">
          <svg class="w-10 h-10 text-gray-700 mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" /></svg>
          <h4 class="text-xs font-bold text-white mb-1">No files found</h4>
          <p class="text-[10px] text-gray-500 font-sans">
            {{ searchQuery ? 'Try adjusting your search query' : 'Drag and drop .jar or .zip files to install them' }}
          </p>
        </div>

        <!-- Dropzone Overlay (Only visible when dragging mods over) -->
        <div v-if="isDragging" class="absolute inset-0 bg-green-500/5 border border-dashed border-green-500/40 rounded-none flex flex-col items-center justify-center z-50 backdrop-blur-[2px] animate-fade-in pointer-events-none">
          <div class="bg-green-600 text-white font-black px-4 py-2 rounded-lg shadow-2xl animate-bounce text-[10px] tracking-wider font-sans">
             DROP FILES TO INSTALL
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue';
import { ask, open } from '@tauri-apps/plugin-dialog';
import { FileService } from '../services/api';
import type { InstanceConfig } from '../services/api';

const props = defineProps<{
  instance: InstanceConfig;
  files: string[];
  isDragging: boolean;
  launchState: { isLaunching: boolean; isPlaying: boolean; progress: number; message: string };
}>();

const emit = defineEmits<{
  (e: 'launch'): void;
  (e: 'delete'): void;
  (e: 'open-folder'): void;
  (e: 'tab-change', tab: string): void;
  (e: 'edit'): void;
  (e: 'delete-file', fileName: string): void;
  (e: 'update-instance', config: Partial<InstanceConfig>): void;
}>();

const activeTab = ref('general');
const searchQuery = ref('');

const saves = ref<string[]>([]);
const screenshots = ref<string[]>([]);

const expandedWorldDatapacks = ref<string | null>(null);
const datapacksMap = ref<Record<string, string[]>>({});

async function loadDatapacks(worldName: string) {
  try {
    datapacksMap.value[worldName] = await FileService.listDatapacks(props.instance.name, worldName);
  } catch (e) {
    console.error('Failed to list datapacks:', e);
    datapacksMap.value[worldName] = [];
  }
}

async function toggleDatapacks(worldName: string) {
  if (expandedWorldDatapacks.value === worldName) {
    expandedWorldDatapacks.value = null;
  } else {
    expandedWorldDatapacks.value = worldName;
    await loadDatapacks(worldName);
  }
}

async function handleDeleteDatapack(worldName: string, datapackName: string) {
  const confirmed = await ask(`Are you sure you want to delete the datapack "${datapackName}"? This cannot be undone!`, {
    title: 'Delete Datapack',
    kind: 'warning'
  });
  if (!confirmed) return;

  try {
    await FileService.deleteDatapack(props.instance.name, worldName, datapackName);
    await loadDatapacks(worldName);
  } catch (e) {
    console.error('Failed to delete datapack:', e);
  }
}

async function handleImportDatapack(worldName: string, type: 'file' | 'directory') {
  try {
    const selected = await open({
      multiple: false,
      directory: type === 'directory',
      filters: type === 'file' ? [{ name: 'Datapacks', extensions: ['zip'] }] : undefined
    });

    if (!selected) return;
    
    const sourcePath = typeof selected === 'string' ? selected : Array.isArray(selected) ? selected[0] : null;
    if (!sourcePath) return;

    await FileService.importDatapack(props.instance.name, worldName, sourcePath);
    await loadDatapacks(worldName);
  } catch (e) {
    console.error('Failed to import datapack:', e);
  }
}

async function loadSavesAndScreenshots() {
  try {
    saves.value = await FileService.listSaves(props.instance.name);
  } catch (e) {
    saves.value = [];
  }
  try {
    screenshots.value = await FileService.listFiles(props.instance.name, 'screenshots');
  } catch (e) {
    screenshots.value = [];
  }
}

// Watch for instance change to reload
watch(() => props.instance.name, () => {
  loadSavesAndScreenshots();
}, { immediate: true });

async function handleOpenLogs() {
  try {
    await FileService.openInSystem(props.instance.name, 'logs/latest.log');
  } catch (e) {
    try {
      await FileService.openInSystem(props.instance.name, 'logs');
    } catch (err) {
      console.error('Failed to open logs:', err);
    }
  }
}

function formatScreenshotName(name: string) {
  const clean = name.replace(/\.png$/i, '');
  // Minecraft screenshots: YYYY-MM-DD_HH.MM.SS
  const regex = /^(\d{4})-(\d{2})-(\d{2})_(\d{2})\.(\d{2})\.(\d{2})$/;
  const match = clean.match(regex);
  if (match) {
    const [_, year, month, day, hour, minute, second] = match;
    const date = new Date(parseInt(year), parseInt(month) - 1, parseInt(day), parseInt(hour), parseInt(minute), parseInt(second));
    
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    
    if (diffMs < 0) {
      return 'Just now';
    }
    
    const diffSecs = Math.floor(diffMs / 1000);
    const diffMins = Math.floor(diffSecs / 60);
    const diffHours = Math.floor(diffMins / 60);
    const diffDays = Math.floor(diffHours / 24);
    
    if (diffSecs < 60) {
      return 'Just now';
    } else if (diffMins < 60) {
      return `${diffMins} ${diffMins === 1 ? 'minute' : 'minutes'} ago`;
    } else if (diffHours < 24) {
      return `${diffHours} ${diffHours === 1 ? 'hour' : 'hours'} ago`;
    } else if (diffDays === 1) {
      return 'Yesterday';
    } else if (diffDays < 7) {
      return `${diffDays} days ago`;
    } else if (diffDays < 30) {
      const weeks = Math.floor(diffDays / 7);
      return `${weeks} ${weeks === 1 ? 'week' : 'weeks'} ago`;
    } else {
      return date.toLocaleDateString(undefined, {
        month: 'short',
        day: 'numeric',
        year: 'numeric'
      });
    }
  }
  return clean.replace(/[-_]/g, ' ');
}

function getScreenshotTimeIdentifier(name: string) {
  const clean = name.replace(/\.png$/i, '');
  const regex = /^(\d{4})-(\d{2})-(\d{2})_(\d{2})\.(\d{2})\.(\d{2})$/;
  const match = clean.match(regex);
  if (match) {
    const [_, , , , hour, minute, second] = match;
    return `${hour}:${minute}:${second}`;
  }
  return '';
}

async function handleDeleteSave(saveName: string) {
  const confirmed = await ask(`Are you sure you want to delete the world "${saveName}"? This cannot be undone!`, {
    title: 'Delete World',
    kind: 'warning'
  });
  if (!confirmed) return;
  
  try {
    await FileService.deleteSave(props.instance.name, saveName);
    saves.value = saves.value.filter(s => s !== saveName);
  } catch (e) {
    console.error('Failed to delete save:', e);
  }
}

async function handleDeleteScreenshot(filename: string) {
  const confirmed = await ask(`Are you sure you want to delete the screenshot "${filename}"? This cannot be undone!`, {
    title: 'Delete Screenshot',
    kind: 'warning'
  });
  if (!confirmed) return;

  try {
    await FileService.deleteFile(props.instance.name, 'screenshots', filename);
    screenshots.value = screenshots.value.filter(s => s !== filename);
  } catch (e) {
    console.error('Failed to delete screenshot:', e);
  }
}

async function handleOpenSubPath(subPath: string) {
  try {
    await FileService.openInSystem(props.instance.name, subPath);
  } catch (e) {
    console.error('Failed to open sub-path:', e);
  }
}

watch(activeTab, (newTab) => {
  searchQuery.value = '';
  emit('tab-change', newTab);
});

const filteredFiles = computed(() => {
  if (!searchQuery.value.trim()) return props.files;
  const q = searchQuery.value.toLowerCase().trim();
  return props.files.filter(f => f.toLowerCase().includes(q));
});

function formatTime(secs: string) {
  const d = new Date(parseInt(secs) * 1000);
  return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
}

function formatDate(iso: string) {
  const ms = parseInt(iso);
  if (isNaN(ms)) return 'Invalid Date';
  return new Date(ms * 1000).toLocaleDateString();
}

function formatFileName(fileName: string) {
  let clean = fileName.replace(/\.(jar|zip)$/i, '');
  const versionRegex = /[-_](v?\d+[\d.\-+]*|mc?\d+[\d.\-+]*|forge|fabric|neoforge|universal)/i;
  const match = clean.match(versionRegex);
  
  let title = clean;
  let version = '';
  
  if (match && match.index !== undefined) {
    title = clean.substring(0, match.index);
    version = clean.substring(match.index + 1);
  }
  
  let formattedTitle = title
    .replace(/[-_]/g, ' ')
    .trim()
    .split(' ')
    .map(w => {
      if (!w) return '';
      return w.charAt(0).toUpperCase() + w.slice(1);
    })
    .filter(Boolean)
    .join(' ');
    
  let formattedVersion = version
    .replace(/[-_]/g, ' ')
    .trim()
    .split(' ')
    .map(w => {
      if (!w) return '';
      if (/^(forge|fabric|neoforge|universal|api)$/i.test(w)) {
        return w.charAt(0).toUpperCase() + w.slice(1).toLowerCase();
      }
      return w;
    })
    .join(' ');
  
  return {
    title: formattedTitle || fileName,
    version: formattedVersion || undefined
  };
}

async function handleLaunch() {
  if (!props.launchState.isLaunching && !props.launchState.isPlaying) {
    emit('launch');
  }
}
</script>

<style scoped>
.fade-enter-active, .fade-leave-active { transition: opacity 0.3s ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
</style>
