<template>
  <div class="fixed inset-0 bg-black/80 backdrop-blur-md flex items-center justify-center z-[100] animate-fade-in" @click.self="$emit('close')">
    <div class="premium-card w-[480px] p-8 flex flex-col gap-5 shadow-2xl relative overflow-hidden">
      <!-- Decoration -->
      <div class="absolute -top-24 -right-24 w-48 h-48 bg-green-500/10 rounded-full blur-3xl"></div>
      
      <header>
        <h3 class="text-2xl font-black text-white tracking-tight">New Instance</h3>
        <p class="text-sm text-gray-500 font-medium">Configure your isolated environment</p>
      </header>

      <!-- Tabs -->
      <div class="flex border-b border-white/5 mb-2 gap-4">
        <button 
          @click="activeTab = 'custom'" 
          :disabled="creating"
          :class="['pb-2 text-[10px] font-black uppercase tracking-widest transition-all relative flex-1 text-center disabled:opacity-50', activeTab === 'custom' ? 'text-white' : 'text-gray-500 hover:text-gray-300']"
        >
          Custom Instance
          <div v-if="activeTab === 'custom'" class="absolute bottom-0 left-0 right-0 h-0.5 bg-green-500 rounded-t-full shadow-[0_0_10px_rgba(34,197,94,0.5)]"></div>
        </button>
        <button 
          @click="activeTab = 'modpack'" 
          :disabled="creating"
          :class="['pb-2 text-[10px] font-black uppercase tracking-widest transition-all relative flex-1 text-center disabled:opacity-50', activeTab === 'modpack' ? 'text-white' : 'text-gray-500 hover:text-gray-300']"
        >
          Import Modpack
          <div v-if="activeTab === 'modpack'" class="absolute bottom-0 left-0 right-0 h-0.5 bg-green-500 rounded-t-full shadow-[0_0_10px_rgba(34,197,94,0.5)]"></div>
        </button>
      </div>

      <!-- Tab Content: Custom Instance -->
      <div v-if="activeTab === 'custom'" class="space-y-4">
        <div class="flex flex-col gap-2">
          <label class="text-[10px] font-black uppercase tracking-widest text-gray-400">Instance Name</label>
          <input v-model="form.name" type="text" placeholder="My Epic Survival" :disabled="creating" class="input-field disabled:opacity-50" />
        </div>

        <div class="grid grid-cols-2 gap-4">
          <div class="flex flex-col gap-2">
            <label class="text-[10px] font-black uppercase tracking-widest text-gray-400">Mod Loader</label>
            <select v-model="form.loader" :disabled="creating" class="input-field cursor-pointer appearance-none disabled:opacity-50">
              <option value="Vanilla">Vanilla</option>
              <option value="Fabric">Fabric</option>
              <option value="Forge">Forge</option>
              <option value="NeoForge">NeoForge</option>
            </select>
          </div>

          <div class="flex flex-col gap-2">
            <label class="text-[10px] font-black uppercase tracking-widest text-gray-400">Version</label>
            <div class="relative">
              <select v-model="selectedMappingIndex" :disabled="loadingVersions || creating" class="input-field w-full cursor-pointer appearance-none disabled:opacity-50">
                <option v-if="loadingVersions" value="-1">Loading...</option>
                <option v-for="(mapping, index) in versions" :key="mapping.minecraft" :value="index">
                  {{ mapping.minecraft }} {{ form.loader !== 'Vanilla' ? '(' + mapping.loader + ')' : '' }}
                </option>
              </select>
              <div v-if="loadingVersions" class="absolute right-3 top-3.5">
                <div class="w-4 h-4 border-2 border-green-500 border-t-transparent rounded-full animate-spin"></div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Tab Content: Import Modpack -->
      <div v-else class="space-y-4">
        <!-- Source Type Selector -->
        <div class="flex gap-2 p-1 bg-white/5 rounded-xl border border-white/5">
          <button 
            @click="importSourceType = 'file'" 
            :disabled="creating"
            type="button"
            :class="['flex-grow py-1.5 rounded-lg text-[9px] font-black uppercase tracking-wider transition-all disabled:opacity-50 text-center', importSourceType === 'file' ? 'bg-white/10 text-white shadow-sm border border-white/5' : 'text-gray-400 hover:text-white']"
          >
            Modpack File (.mrpack, .zip)
          </button>
          <button 
            @click="importSourceType = 'folder'" 
            :disabled="creating"
            type="button"
            :class="['flex-grow py-1.5 rounded-lg text-[9px] font-black uppercase tracking-wider transition-all disabled:opacity-50 text-center', importSourceType === 'folder' ? 'bg-white/10 text-white shadow-sm border border-white/5' : 'text-gray-400 hover:text-white']"
          >
            Unzipped Folder
          </button>
        </div>

        <div class="flex flex-col gap-2">
          <label class="text-[10px] font-black uppercase tracking-widest text-gray-400">
            {{ importSourceType === 'file' ? 'Modpack File' : 'Modpack Folder' }}
          </label>
          <div class="flex gap-2">
            <input 
              :value="modpackPath" 
              type="text" 
              :placeholder="importSourceType === 'file' ? 'Select a .mrpack or .zip file...' : 'Select unzipped CurseForge folder...'" 
              readonly 
              class="input-field flex-1 opacity-70" 
            />
            <button @click="handleBrowseSource" :disabled="creating" class="px-4 py-2 bg-white/5 border border-white/10 rounded-lg hover:bg-white/10 active:scale-95 text-xs font-bold text-white transition-all disabled:opacity-50 disabled:active:scale-100">
              Browse
            </button>
          </div>
        </div>

        <div class="flex flex-col gap-2">
          <label class="text-[10px] font-black uppercase tracking-widest text-gray-400">Instance Name</label>
          <input v-model="form.name" type="text" placeholder="My Zombie Apocalypse" :disabled="creating" class="input-field disabled:opacity-50" />
        </div>
      </div>

      <!-- Shared Configuration -->
      <div class="grid grid-cols-2 gap-4">
        <div class="flex flex-col gap-2">
          <label class="text-[10px] font-black uppercase tracking-widest text-gray-400">Username</label>
          <input v-model="form.username" type="text" placeholder="Player" :disabled="creating" class="input-field disabled:opacity-50" />
        </div>
        <div class="flex flex-col gap-2">
          <label class="text-[10px] font-black uppercase tracking-widest text-gray-400">Max RAM</label>
          <select v-model="form.max_memory" :disabled="creating" class="input-field cursor-pointer appearance-none disabled:opacity-50">
            <option value="2G">2 GB</option>
            <option value="4G">4 GB</option>
            <option value="6G">6 GB</option>
            <option value="8G">8 GB</option>
            <option value="12G">12 GB</option>
          </select>
        </div>
      </div>

      <!-- Progress Tracking for Modpack Import -->
      <div v-if="creating && activeTab === 'modpack'" class="space-y-2 mt-2">
        <div class="flex justify-between text-[10px] font-mono text-green-400">
          <span class="truncate pr-4">{{ progressMessage || 'Importing modpack...' }}</span>
          <span class="shrink-0">{{ progressPercent }}%</span>
        </div>
        <div class="w-full bg-gray-900 h-1.5 rounded-full overflow-hidden border border-white/5 relative">
          <div class="bg-green-500 h-full transition-all duration-300 relative shadow-[0_0_10px_rgba(34,197,94,0.5)]" :style="{ width: progressPercent + '%' }">
            <div class="absolute inset-0 bg-white/20 animate-pulse"></div>
          </div>
        </div>
      </div>

      <!-- Actions -->
      <div class="flex gap-3 mt-4">
        <button @click="$emit('close')" :disabled="creating" class="flex-1 btn-secondary disabled:opacity-50">
          Cancel
        </button>
        <button 
          @click="handleCreate" 
          :disabled="!form.name || (activeTab === 'custom' && selectedMappingIndex === -1) || (activeTab === 'modpack' && !modpackPath) || creating" 
          class="flex-1 btn-primary disabled:opacity-40 disabled:active:scale-100 flex items-center justify-center gap-2"
        >
          <span v-if="creating" class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></span>
          {{ creating ? (activeTab === 'modpack' ? 'Importing...' : 'Creating...') : 'Create' }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';
import { open, message } from '@tauri-apps/plugin-dialog';
import { listen } from '@tauri-apps/api/event';
import { ModpackService } from '../services/api';
import type { LoaderVersionMapping } from '../services/api';

const props = defineProps<{
  versions: LoaderVersionMapping[];
  loadingVersions: boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'create', data: any): void;
  (e: 'created-instance', instance: any): void;
  (e: 'change-loader', loader: string): void;
}>();

const creating = ref(false);
const activeTab = ref('custom');
const selectedMappingIndex = ref(-1);
const importSourceType = ref<'file' | 'folder'>('file');
const modpackPath = ref('');

watch(importSourceType, () => {
  modpackPath.value = '';
});

const progressPercent = ref(0);
const progressMessage = ref('');

const form = ref({
  name: '',
  version_id: '',
  loader: 'Vanilla',
  loader_version: '',
  username: 'IsoPlayer',
  max_memory: '4G',
});

let unlistenProgress: (() => void) | null = null;

onMounted(async () => {
  if (props.versions.length > 0) {
    selectedMappingIndex.value = 0;
  }

  unlistenProgress = await listen<{ instance_name: string, progress: number, message: string }>('modpack-progress', (event) => {
    if (event.payload.instance_name === form.value.name) {
      progressPercent.value = event.payload.progress;
      progressMessage.value = event.payload.message;
    }
  });
});

onUnmounted(() => {
  if (unlistenProgress) {
    unlistenProgress();
  }
});

watch(() => form.value.loader, (newLoader) => {
  selectedMappingIndex.value = -1;
  form.value.version_id = '';
  form.value.loader_version = '';
  emit('change-loader', newLoader);
});

watch(() => props.versions, (newVersions) => {
  if (newVersions.length > 0) {
    selectedMappingIndex.value = 0;
    form.value.version_id = newVersions[0].minecraft;
    form.value.loader_version = newVersions[0].loader;
  } else {
    selectedMappingIndex.value = -1;
    form.value.version_id = '';
    form.value.loader_version = '';
  }
});

watch(selectedMappingIndex, (newIdx) => {
  if (newIdx !== -1 && props.versions[newIdx]) {
    form.value.version_id = props.versions[newIdx].minecraft;
    form.value.loader_version = props.versions[newIdx].loader;
  }
});

async function handleBrowseSource() {
  try {
    let selected;
    if (importSourceType.value === 'file') {
      selected = await open({
        multiple: false,
        directory: false,
        title: 'Select Modpack File (.mrpack, .zip)',
        filters: [{ name: 'Modpacks', extensions: ['mrpack', 'zip'] }]
      });
    } else {
      selected = await open({
        multiple: false,
        directory: true,
        title: 'Select CurseForge Modpack Directory'
      });
    }
    
    if (selected && typeof selected === 'string') {
      modpackPath.value = selected;
      
      // Auto-fill instance name based on file or folder name
      const parts = selected.split(/[/\\]/);
      let lastPart = parts[parts.length - 1] || '';
      if (importSourceType.value === 'file') {
        lastPart = lastPart.replace(/\.(mrpack|zip)$/i, '');
      }
      if (lastPart) {
        // Replace spaces or special characters to keep it nice
        form.value.name = lastPart.replace(/[^a-zA-Z0-9_\-\s]/g, '').trim();
      }
    }
  } catch (e) {
    console.error('Failed to open selection dialog:', e);
  }
}

async function handleCreate() {
  creating.value = true;
  progressPercent.value = 0;
  progressMessage.value = '';
  
  try {
    if (activeTab.value === 'custom') {
      await emit('create', { ...form.value });
    } else {
      if (!modpackPath.value) {
        throw new Error(importSourceType.value === 'file' ? "Please select a modpack file first." : "Please select a modpack folder first.");
      }
      
      const created = await ModpackService.importModpack(
        modpackPath.value,
        form.value.name,
        form.value.username,
        form.value.max_memory
      );
      
      emit('created-instance', created);
    }
  } catch (e) {
    await message(String(e), { title: 'Import Error', kind: 'error' });
  } finally {
    creating.value = false;
  }
}
</script>
