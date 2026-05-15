<template>
  <aside class="w-72 bg-[var(--bg-sidebar)] border-r border-[var(--border-color)] flex flex-col h-full z-10">
    <header class="p-6 border-b border-[var(--border-color)]">
      <h1 class="text-2xl font-black text-white tracking-tighter flex items-center gap-2">
        <div class="w-8 h-8 bg-green-600 rounded-lg flex items-center justify-center shadow-lg shadow-green-900/40">
          <svg class="w-5 h-5 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" />
          </svg>
        </div>
        IsoCraft
      </h1>
      <p class="text-[10px] font-bold text-[var(--text-muted)] mt-1 uppercase tracking-[0.2em]">Total Isolation</p>
    </header>

    <div class="flex-1 overflow-y-auto p-4 space-y-3 custom-scrollbar">
      <div v-if="instances.length === 0" class="text-center py-12 px-4">
        <div class="w-12 h-12 bg-gray-800/30 rounded-full flex items-center justify-center mx-auto mb-3 border border-gray-700/30">
          <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v3m0 0v3m0-3h3m-3 0H9m12 0a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        </div>
        <p class="text-sm text-gray-500 font-medium">No instances found.<br/>Create one to start.</p>
      </div>

      <button
        v-for="inst in instances"
        :key="inst.name"
        @click="$emit('select', inst)"
        :class="[
          'w-full text-left p-4 rounded-2xl transition-all duration-300 group relative overflow-hidden',
          selectedName === inst.name
            ? 'bg-green-600/10 border border-green-500/30 shadow-lg shadow-green-500/5'
            : 'bg-black/20 border border-transparent hover:border-gray-700/50 hover:bg-black/40'
        ]"
      >
        <!-- Active Indicator -->
        <div v-if="selectedName === inst.name" class="absolute left-0 top-0 bottom-0 w-1 bg-green-500"></div>

        <div class="font-bold text-sm" :class="selectedName === inst.name ? 'text-white' : 'text-gray-300 group-hover:text-white'">
          {{ inst.name }}
        </div>
        <div class="text-[10px] font-bold text-gray-500 mt-2 flex items-center gap-2">
          <span class="bg-gray-800/80 px-2 py-0.5 rounded border border-gray-700 font-mono text-gray-400">{{ inst.version_id }}</span>
          <span class="uppercase tracking-wider">{{ inst.loader }}</span>
        </div>
      </button>
    </div>

    <div class="p-4 border-t border-[var(--border-color)] bg-black/10">
      <button
        @click="$emit('request-create')"
        class="w-full btn-primary flex items-center justify-center gap-2 group"
      >
        <svg class="w-5 h-5 transition-transform group-hover:rotate-90" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M12 4v16m8-8H4" />
        </svg>
        New Instance
      </button>
    </div>
  </aside>
</template>

<script setup lang="ts">
import type { InstanceConfig } from '../services/api';

defineProps<{
  instances: InstanceConfig[];
  selectedName?: string;
}>();

defineEmits<{
  (e: 'select', instance: InstanceConfig): void;
  (e: 'request-create'): void;
}>();
</script>
