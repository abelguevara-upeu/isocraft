<template>
  <div class="flex-1 flex flex-col h-full overflow-hidden bg-[var(--bg-main)] animate-fade-in">
    <!-- Banner / Header -->
    <header class="relative h-72 flex-shrink-0 overflow-hidden border-b border-[var(--border-color)]">
      <!-- Dynamic Background Gradient -->
      <div class="absolute inset-0 bg-gradient-to-br from-green-900/20 via-[#0a0d12] to-[#0a0d12]"></div>
      
      <!-- Content Overlay -->
      <div class="relative z-10 h-full flex flex-col justify-end p-12">
        <div class="flex justify-between items-end w-full">
          <div>
            <h2 class="text-6xl font-black text-white tracking-tighter drop-shadow-2xl mb-4">{{ instance.name }}</h2>
            <div class="flex items-center gap-6">
              <div class="flex items-center gap-2 px-3 py-1 bg-white/5 border border-white/10 rounded-lg backdrop-blur-sm shadow-inner">
                <span class="text-[10px] font-black uppercase tracking-widest text-gray-500">Loader</span>
                <span class="text-sm font-bold text-green-500">{{ instance.loader }} {{ instance.loader_version !== instance.version_id ? instance.loader_version : '' }}</span>
              </div>
              <div class="flex items-center gap-2 px-3 py-1 bg-white/5 border border-white/10 rounded-lg backdrop-blur-sm shadow-inner">
                <span class="text-[10px] font-black uppercase tracking-widest text-gray-400">Version</span>
                <span class="text-sm font-bold text-blue-400 font-mono">{{ instance.version_id }}</span>
              </div>
              <div class="flex items-center gap-2 px-3 py-1 bg-white/5 border border-white/10 rounded-lg backdrop-blur-sm shadow-inner">
                <span class="text-[10px] font-black uppercase tracking-widest text-gray-500">RAM</span>
                <span class="text-sm font-bold text-purple-400">{{ instance.max_memory }}</span>
              </div>
            </div>
          </div>

          <div class="flex gap-3 mb-1">
            <button @click="$emit('open-folder')" class="btn-secondary group px-5">
              <svg class="w-5 h-5 group-hover:scale-110 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
              </svg>
            </button>
            <button @click="$emit('delete')" class="btn-secondary group border-red-900/30 hover:border-red-500/50 hover:bg-red-500/10 text-red-500/80 hover:text-red-500 px-5 transition-all">
              <svg class="w-5 h-5 group-hover:rotate-12 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
              </svg>
            </button>
          </div>
        </div>
      </div>
    </header>

    <!-- Navigation Tabs -->
    <nav class="flex gap-10 px-12 pt-6 border-b border-[var(--border-color)] bg-black/20">
      <button 
        v-for="tab in ['general', 'mods', 'resourcepacks']" 
        :key="tab"
        @click="activeTab = tab"
        :class="[
          'pb-4 text-xs font-black uppercase tracking-[0.2em] transition-all relative',
          activeTab === tab ? 'text-white' : 'text-gray-500 hover:text-gray-300'
        ]"
      >
        {{ tab }}
        <div v-if="activeTab === tab" class="absolute bottom-0 left-0 right-0 h-1 bg-green-500 rounded-t-full shadow-[0_0_10px_rgba(34,197,94,0.5)]"></div>
      </button>
    </nav>

    <!-- Content Area -->
    <div class="flex-1 overflow-y-auto p-12 custom-scrollbar flex flex-col">
      
      <!-- Tab Content: General -->
      <div v-if="activeTab === 'general'" class="space-y-8 max-w-3xl">
        <section class="premium-card p-8 bg-gradient-to-br from-gray-800/40 to-transparent">
          <h3 class="text-xs font-black uppercase tracking-[0.2em] text-gray-500 mb-6 flex items-center gap-3">
            <svg class="w-4 h-4 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" /></svg>
            Isolated Environment
          </h3>
          <div class="bg-black/60 rounded-2xl p-6 border border-white/5 font-mono text-sm text-gray-400 break-all shadow-inner">
            <span class="text-green-600/80 font-bold">$ </span>~/.isocraft/instances/{{ instance.name }}
          </div>
          <p class="mt-6 text-sm text-gray-500 leading-relaxed">
            This instance runs in a completely separate directory. All your worlds, mods, and configurations are isolated from other versions and your main Minecraft installation.
          </p>
        </section>

        <section class="grid grid-cols-2 gap-6">
          <div class="premium-card p-6 border-blue-500/20 bg-blue-500/5">
             <div class="text-[10px] font-black uppercase tracking-widest text-blue-400 mb-2">Last Played</div>
             <div class="text-lg font-bold text-white">{{ instance.last_played ? formatTime(instance.last_played) : 'Never' }}</div>
          </div>
          <div class="premium-card p-6 border-purple-500/20 bg-purple-500/5">
             <div class="text-[10px] font-black uppercase tracking-widest text-purple-400 mb-2">Created On</div>
             <div class="text-lg font-bold text-white">{{ formatDate(instance.created_at) }}</div>
          </div>
        </section>
      </div>

      <!-- Tab Content: Files -->
      <div v-else class="flex-1 flex flex-col">
        <div 
          class="flex-1 premium-card border-2 border-dashed flex flex-col relative transition-all"
          :class="isDragging ? 'border-green-500 bg-green-500/5' : 'border-gray-800'"
        >
          <div v-if="files.length > 0" class="p-6 grid grid-cols-2 lg:grid-cols-3 gap-4 overflow-y-auto">
            <div v-for="file in files" :key="file" class="bg-black/30 border border-white/5 p-4 rounded-xl flex items-center gap-4 hover:border-gray-600 transition-all group shadow-lg">
               <div class="w-10 h-10 bg-gray-900 rounded-lg flex items-center justify-center flex-shrink-0 group-hover:bg-gray-800 transition-colors">
                  <svg class="w-5 h-5 text-blue-500" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z" /></svg>
               </div>
               <span class="text-sm font-semibold text-gray-300 truncate" :title="file">{{ file }}</span>
            </div>
          </div>
          <div v-else class="flex-1 flex flex-col items-center justify-center p-12 text-center opacity-50">
            <svg class="w-16 h-16 text-gray-700 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12" /></svg>
            <h4 class="text-xl font-bold text-white mb-2">Drop files here</h4>
            <p class="text-sm text-gray-500">Drag and drop .jar or .zip files to install them in <span class="text-white">{{ activeTab }}</span></p>
          </div>
          
          <div v-if="isDragging" class="absolute inset-0 flex items-center justify-center z-50 pointer-events-none">
            <div class="bg-green-600 text-white font-black px-8 py-4 rounded-2xl shadow-2xl animate-bounce">
              RELEASE TO INSTALL
            </div>
          </div>
        </div>
      </div>

      <!-- Launch Area -->
      <div class="mt-auto pt-10">
        <!-- Progress Bar -->
        <transition name="fade">
          <div v-if="launchState.isLaunching" class="mb-6 space-y-3">
            <div class="flex justify-between items-center text-[10px] font-black uppercase tracking-widest">
              <span class="text-green-500 animate-pulse">{{ launchState.message }}</span>
              <span class="text-white">{{ launchState.progress }}%</span>
            </div>
            <div class="w-full bg-gray-900 rounded-full h-4 p-1 border border-white/5 overflow-hidden">
              <div 
                class="bg-gradient-to-r from-green-600 to-green-400 h-full rounded-full transition-all duration-500 relative shadow-[0_0_15px_rgba(34,197,94,0.3)]"
                :style="{ width: launchState.progress + '%' }"
              >
                <div class="absolute inset-0 bg-white/20 animate-pulse"></div>
              </div>
            </div>
          </div>
        </transition>

        <button 
          @click="handleLaunch"
          :disabled="launchState.isLaunching"
          class="w-full py-6 rounded-2xl text-2xl font-black tracking-tighter flex items-center justify-center gap-4 transition-all relative overflow-hidden group shadow-2xl"
          :class="launchState.isPlaying 
            ? 'bg-gray-800 text-green-500 border border-green-500/20' 
            : 'bg-green-600 hover:bg-green-500 text-white hover:scale-[1.02] active:scale-95 shadow-green-900/30 hover:shadow-green-500/20'"
        >
          <template v-if="launchState.isPlaying">
             <div class="w-3 h-3 bg-green-500 rounded-full animate-ping"></div>
             GAME RUNNING
          </template>
          <template v-else-if="launchState.isLaunching">
             <svg class="w-8 h-8 animate-spin" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" /></svg>
             PREPARING...
          </template>
          <template v-else>
             <svg class="w-8 h-8 transition-transform group-hover:scale-125" fill="currentColor" viewBox="0 0 24 24"><path d="M8 5v14l11-7z"/></svg>
             PLAY NOW
          </template>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
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
}>();

const activeTab = ref('general');

watch(activeTab, (newTab) => {
  emit('tab-change', newTab);
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
