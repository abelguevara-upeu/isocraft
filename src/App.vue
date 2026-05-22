<template>
  <div class="h-screen w-full flex overflow-hidden font-sans bg-[var(--bg-main)]">
    
    <!-- Navigation Sidebar -->
    <Sidebar 
      :instances="instances" 
      :selected-name="selectedInstance?.name"
      @select="selectInstance"
      @request-create="showCreateModal = true"
    />

    <!-- Main Content Area -->
    <main class="flex-1 relative flex flex-col min-w-0">
      <InstanceDetails 
        v-if="selectedInstance"
        :instance="selectedInstance"
        :files="filesList"
        :is-dragging="isDragging"
        :launch-state="getLaunchState(selectedInstance.name)"
        @launch="launchGame"
        @delete="handleDelete"
        @open-folder="openInstanceFolder"
        @tab-change="handleTabChange"
        @edit="openEditModal"
        @delete-file="deleteFile"
        @update-instance="updateInstance"
      />

      <!-- Empty State -->
      <div v-else class="flex-1 flex flex-col items-center justify-center space-y-6 animate-fade-in">
        <div class="w-32 h-32 bg-white/5 rounded-[2.5rem] flex items-center justify-center border border-white/10 shadow-2xl">
          <svg class="w-16 h-16 text-gray-700" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" />
          </svg>
        </div>
        <div class="text-center">
          <h2 class="text-2xl font-black text-white tracking-tight">Welcome to IsoCraft</h2>
          <p class="text-gray-500 font-medium mt-1">Select an instance or create a new one to begin</p>
        </div>
      </div>
    </main>

    <!-- Modals -->
    <CreateInstanceModal 
      v-if="showCreateModal"
      :versions="availableVersions"
      :loading-versions="isLoadingVersions"
      @close="showCreateModal = false"
      @create="createInstance"
      @created-instance="handleInstanceCreated"
      @change-loader="loadVersions"
    />

    <EditInstanceModal 
      v-if="showEditModal && selectedInstance"
      :instance="selectedInstance"
      :versions="availableVersions"
      :loading-versions="isLoadingVersions"
      @close="showEditModal = false"
      @save="updateInstance"
      @change-loader="loadVersions"
    />

    <!-- Launch Warning Toast -->
    <Transition name="toast">
      <div
        v-if="launchWarning"
        class="fixed bottom-6 left-1/2 -translate-x-1/2 z-[200] w-[520px] max-w-[90vw]"
      >
        <div class="flex items-start gap-3 bg-[#1a1208] border border-amber-500/40 rounded-2xl px-5 py-4 shadow-2xl backdrop-blur-md">
          <div class="mt-0.5 text-amber-400 shrink-0">
            <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126ZM12 15.75h.007v.008H12v-.008Z"/>
            </svg>
          </div>
          <div class="flex-1 min-w-0">
            <p class="text-amber-300 font-bold text-sm">Known Issue — {{ launchWarning.version_id }}</p>
            <p class="text-amber-200/70 text-xs mt-1 leading-relaxed">{{ launchWarning.message }}</p>
          </div>
          <button @click="launchWarning = null" class="text-amber-500/60 hover:text-amber-300 transition-colors shrink-0 mt-0.5">
            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
              <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12"/>
            </svg>
          </button>
        </div>
      </div>
    </Transition>

  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { ask, message } from '@tauri-apps/plugin-dialog';

// Components
import Sidebar from './components/Sidebar.vue';
import InstanceDetails from './components/InstanceDetails.vue';
import CreateInstanceModal from './components/CreateInstanceModal.vue';
import EditInstanceModal from './components/EditInstanceModal.vue';

// Services
import { InstanceService, LauncherService, FileService, type InstanceConfig, type LoaderVersionMapping } from './services/api';

// --- State ---
const instances = ref<InstanceConfig[]>([]);
const selectedInstance = ref<InstanceConfig | null>(null);
const availableVersions = ref<LoaderVersionMapping[]>([]);
const isLoadingVersions = ref(true);
const showCreateModal = ref(false);
const showEditModal = ref(false);
const activeTab = ref('general');
const filesList = ref<string[]>([]);
const isDragging = ref(false);

interface LaunchState {
  isLaunching: boolean;
  isPlaying: boolean;
  progress: number;
  message: string;
}
const launchStates = ref<Record<string, LaunchState>>({});
const launchWarning = ref<{ instance_name: string; version_id: string; message: string } | null>(null);

// --- Helpers ---
function getLaunchState(name: string): LaunchState {
  if (!launchStates.value[name]) {
    launchStates.value[name] = { isLaunching: false, isPlaying: false, progress: 0, message: '' };
  }
  return launchStates.value[name];
}

async function refreshInstances() {
  instances.value = await InstanceService.listInstances();
  if (instances.value.length > 0 && !selectedInstance.value) {
    selectedInstance.value = instances.value[0];
  }
}

async function loadVersions(loader: string = 'Vanilla') {
  isLoadingVersions.value = true;
  availableVersions.value = []; // Clear immediately to avoid stale data from previous loader
  try {
    availableVersions.value = await InstanceService.getVersions(loader);
  } catch (e) {
    console.error("Failed to load versions:", e);
  } finally {
    isLoadingVersions.value = false;
  }
}

async function loadFiles() {
  if (!selectedInstance.value || activeTab.value === 'general') return;
  try {
    filesList.value = await FileService.listFiles(selectedInstance.value.name, activeTab.value);
  } catch (e) {
    console.error("Failed to load files:", e);
  }
}

// --- Event Handlers ---
// Store unlisten functions to avoid accumulating duplicate listeners across re-mounts
const unlisteners: Array<() => void> = [];

onMounted(async () => {
  await refreshInstances();
  await loadVersions();

  // Listen for progress
  unlisteners.push(await listen<{ instance_name: string, progress: number, message: string }>('jre-progress', (event) => {
    const st = getLaunchState(event.payload.instance_name);
    st.progress = event.payload.progress;
    st.message = event.payload.message;
  }));

  unlisteners.push(await listen<{ instance_name: string }>('game-launched', (event) => {
    const st = getLaunchState(event.payload.instance_name);
    st.isLaunching = false;
    st.isPlaying = true;
  }));

  unlisteners.push(await listen<string>('game-closed', (event) => {
    const st = getLaunchState(event.payload);
    st.isPlaying = false;
  }));

  unlisteners.push(await listen<{ instance_name: string; version_id: string; message: string }>('launch-warning', (event) => {
    launchWarning.value = event.payload;
    // Auto-dismiss after 12 seconds
    setTimeout(() => { launchWarning.value = null; }, 12000);
  }));

  // Drag & Drop — each dropped item can be a single file or a folder
  unlisteners.push(await listen<{ paths: string[] }>('tauri://drag-drop', async (event) => {
    isDragging.value = false;
    if (activeTab.value === 'general' || !selectedInstance.value) return;

    let totalImported = 0;
    let totalSkipped = 0;

    for (const path of event.payload.paths) {
      try {
        const result = await FileService.importPath(selectedInstance.value.name, activeTab.value, path);
        totalImported += result.imported;
        totalSkipped += result.skipped;
      } catch (e) {
        console.error("Import error:", e);
      }
    }

    if (totalImported > 0 || totalSkipped > 0) {
      await loadFiles();
      const parts: string[] = [];
      if (totalImported > 0) parts.push(`${totalImported} file${totalImported > 1 ? 's' : ''} added`);
      if (totalSkipped > 0) parts.push(`${totalSkipped} already existed (skipped)`);
      await message(parts.join(' · '), { title: 'Import Complete', kind: 'info' });
    }
  }));

  unlisteners.push(await listen('tauri://drag-enter', () => { if (activeTab.value !== 'general') isDragging.value = true; }));
  unlisteners.push(await listen('tauri://drag-leave', () => { isDragging.value = false; }));
});

onUnmounted(() => {
  // Clean up all Tauri event listeners to prevent duplicates on re-mount
  for (const unlisten of unlisteners) {
    unlisten();
  }
});

async function selectInstance(instance: InstanceConfig) {
  selectedInstance.value = instance;
  activeTab.value = 'general';
  filesList.value = [];
}

async function createInstance(data: any) {
  try {
    const created = await InstanceService.createInstance(data);
    instances.value.unshift(created);
    selectedInstance.value = created;
    showCreateModal.value = false;
  } catch (e) {
    await message(String(e), { title: 'Creation Error', kind: 'error' });
  }
}

async function handleInstanceCreated(created: InstanceConfig) {
  instances.value.unshift(created);
  selectedInstance.value = created;
  showCreateModal.value = false;
}

function openEditModal() {
  showEditModal.value = true;
}

async function updateInstance(data: any) {
  if (!selectedInstance.value) return;
  const currentName = selectedInstance.value.name;
  try {
    const updated = await InstanceService.updateInstance(currentName, data);
    
    // Update local list
    instances.value = instances.value.map(i => i.name === currentName ? updated : i);
    selectedInstance.value = updated;
    showEditModal.value = false;
  } catch (e) {
    await message(String(e), { title: 'Update Error', kind: 'error' });
  }
}

async function deleteFile(fileName: string) {
  if (!selectedInstance.value) return;
  const name = selectedInstance.value.name;
  
  const confirmed = await ask(`Are you sure you want to delete the file "${fileName}"? This cannot be undone.`, {
    title: 'Confirm Deletion',
    kind: 'warning'
  });

  if (!confirmed) return;

  try {
    await FileService.deleteFile(name, activeTab.value, fileName);
    await loadFiles(); // Reload file list
  } catch (e) {
    await message(String(e), { title: 'Delete Error', kind: 'error' });
  }
}

async function launchGame() {
  if (!selectedInstance.value) return;
  const name = selectedInstance.value.name;
  const st = getLaunchState(name);

  if (st.isLaunching || st.isPlaying) return;

  st.isLaunching = true;
  st.progress = 0;
  st.message = "Preparing manifest...";

  try {
    await LauncherService.launchInstance(name);
  } catch (e) {
    st.isLaunching = false;
    await message(String(e), { title: 'Launch Error', kind: 'error' });
  } finally {
    await refreshInstances(); // Update last_played
  }
}

async function openInstanceFolder() {
  if (!selectedInstance.value) return;
  try {
    await InstanceService.openFolder(selectedInstance.value.name);
  } catch (e) {
    await message(String(e), { title: 'Error Opening Folder', kind: 'error' });
  }
}

async function handleTabChange(tab: string) {
  activeTab.value = tab;
  await loadFiles();
}

async function handleDelete() {
  if (!selectedInstance.value) return;
  const name = selectedInstance.value.name;
  
  const confirmed = await ask(`Are you sure you want to delete "${name}"? All data (saves, mods, configs) will be lost permanently.`, {
    title: 'Confirm Deletion',
    kind: 'warning'
  });

  if (!confirmed) return;

  try {
    await InstanceService.deleteInstance(name);
    instances.value = instances.value.filter(i => i.name !== name);
    selectedInstance.value = instances.value.length > 0 ? instances.value[0] : null;
  } catch (e) {
    await message(String(e), { title: 'Delete Error', kind: 'error' });
  }
}
</script>
