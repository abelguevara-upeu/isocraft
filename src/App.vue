<template>
  <div class="h-screen w-full bg-[#0d1117] text-gray-200 flex font-sans">

    <!-- Sidebar: Instancias -->
    <aside class="w-64 bg-[#161b22] border-r border-gray-700/50 flex flex-col">
      <header class="p-5 border-b border-gray-700/50">
        <h1 class="text-xl font-extrabold text-white tracking-tight">IsoCraft</h1>
        <p class="text-xs text-gray-500 mt-0.5">Aislamiento Total</p>
      </header>

      <div class="flex-1 overflow-y-auto p-3 space-y-2">
        <div v-if="instances.length === 0" class="text-center text-gray-600 text-sm py-8">
          Sin instancias.<br/>Crea una para empezar.
        </div>

        <button
          v-for="inst in instances"
          :key="inst.name"
          @click="selectInstance(inst)"
          :class="[
            'w-full text-left p-3 rounded-xl transition-all duration-200 group',
            selectedInstance?.name === inst.name
              ? 'bg-green-600/20 border border-green-500/40 shadow-lg shadow-green-500/5'
              : 'bg-[#0d1117]/50 border border-transparent hover:border-gray-600/50 hover:bg-[#0d1117]'
          ]"
        >
          <div class="font-semibold text-sm text-white truncate">{{ inst.name }}</div>
          <div class="text-xs text-gray-400 mt-1 flex items-center gap-2">
            <span class="bg-gray-800 px-1.5 py-0.5 rounded text-[10px] font-mono">{{ inst.version_id }}</span>
            <span class="text-gray-600">{{ inst.max_memory }}</span>
          </div>
        </button>
      </div>

      <div class="p-3 border-t border-gray-700/50">
        <button
          @click="showCreateModal = true"
          class="w-full py-2.5 rounded-xl text-sm font-semibold bg-blue-600 hover:bg-blue-500 text-white transition-all flex items-center justify-center gap-2"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          Nueva Instancia
        </button>
      </div>
    </aside>

    <!-- Main Content -->
    <main class="flex-1 flex flex-col bg-[#0a0d12] relative overflow-hidden">

      <!-- Estado: Ninguna instancia seleccionada -->
      <div v-if="!selectedInstance" class="flex-1 flex flex-col items-center justify-center space-y-4">
        <div class="w-20 h-20 mx-auto bg-gray-800/50 rounded-2xl flex items-center justify-center border border-gray-700/50">
          <svg class="w-10 h-10 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" />
          </svg>
        </div>
        <p class="text-gray-500 text-sm font-medium">Selecciona o crea una instancia para empezar</p>
      </div>

      <!-- Estado: Instancia seleccionada -->
      <div v-else class="flex-1 flex flex-col relative">
        <!-- Banner superior / Cabecera -->
        <div class="h-64 bg-gradient-to-br from-green-900/30 via-[#0a0d12] to-[#0a0d12] border-b border-gray-800 relative flex flex-col justify-end p-10">
          
          <!-- Botones de Acción Superiores -->
          <div class="absolute top-6 right-6 flex gap-3">
            <button
              @click="abrirCarpeta(selectedInstance.name)"
              class="px-4 py-2 flex items-center gap-2 rounded-lg bg-black/40 border border-gray-700 hover:bg-gray-800 hover:border-gray-500 text-gray-300 hover:text-white transition-all text-sm font-medium backdrop-blur-sm"
              title="Abrir Carpeta"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
              </svg>
              Carpeta
            </button>
            <button
              @click="confirmDelete"
              class="px-4 py-2 flex items-center gap-2 rounded-lg bg-black/40 border border-gray-700 hover:bg-red-900/50 hover:border-red-500/50 text-gray-300 hover:text-red-400 transition-all text-sm font-medium backdrop-blur-sm"
              title="Eliminar instancia"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
              </svg>
              Eliminar
            </button>
          </div>

          <!-- Info de la Instancia -->
          <div class="relative z-10">
            <h2 class="text-5xl font-extrabold text-white tracking-tight drop-shadow-md">{{ selectedInstance.name }}</h2>
            <div class="flex items-center gap-4 mt-4 text-sm text-gray-400 font-medium">
              <span class="bg-gray-800 text-gray-200 px-3 py-1 rounded-md border border-gray-700 font-mono shadow-sm">
                {{ selectedInstance.version_id }}
              </span>
              <span class="flex items-center gap-1.5">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" /></svg>
                {{ selectedInstance.max_memory }} RAM
              </span>
              <span class="flex items-center gap-1.5">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" /></svg>
                {{ selectedInstance.username }}
              </span>
            </div>
          </div>
        </div>

        <!-- Pestañas -->
        <div class="flex gap-8 px-10 border-b border-gray-800/80 bg-[#0a0d12] pt-4">
          <button @click="activeTab = 'general'" :class="{'text-white border-b-2 border-green-500': activeTab === 'general', 'text-gray-500 hover:text-gray-300': activeTab !== 'general'}" class="pb-3 text-sm font-bold uppercase tracking-wider transition-colors">General</button>
          <button @click="activeTab = 'mods'" :class="{'text-white border-b-2 border-green-500': activeTab === 'mods', 'text-gray-500 hover:text-gray-300': activeTab !== 'mods'}" class="pb-3 text-sm font-bold uppercase tracking-wider transition-colors">Mods</button>
          <button @click="activeTab = 'resourcepacks'" :class="{'text-white border-b-2 border-green-500': activeTab === 'resourcepacks', 'text-gray-500 hover:text-gray-300': activeTab !== 'resourcepacks'}" class="pb-3 text-sm font-bold uppercase tracking-wider transition-colors">Resourcepacks</button>
        </div>

        <!-- Contenido principal y controles de lanzamiento -->
        <div class="flex-1 p-10 flex flex-col justify-between bg-[#0a0d12] relative">
          
          <!-- Tab: General -->
          <div v-if="activeTab === 'general'" class="max-w-2xl">
            <div class="bg-[#12161d] rounded-xl p-5 border border-gray-800/80 shadow-sm">
              <h3 class="text-xs uppercase tracking-widest text-gray-500 mb-3 font-bold flex items-center gap-2">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" /></svg>
                Directorio Aislado
              </h3>
              <div class="text-sm text-gray-400 font-mono bg-black/40 p-3 rounded-lg border border-gray-800 break-all select-all">
                {{ baseDataDir }}/instances/{{ selectedInstance.name }}/
              </div>
              <div class="mt-4">
                <span class="text-[10px] uppercase tracking-widest text-gray-500 mb-2 block font-semibold">Ejemplos de contenido independiente:</span>
                <div class="flex flex-wrap gap-2">
                  <span class="text-xs font-mono bg-gray-800/80 text-gray-400 px-2.5 py-1 rounded border border-gray-700/50">saves/</span>
                  <span class="text-xs font-mono bg-gray-800/80 text-gray-400 px-2.5 py-1 rounded border border-gray-700/50">mods/</span>
                  <span class="text-xs font-mono bg-gray-800/80 text-gray-400 px-2.5 py-1 rounded border border-gray-700/50">resourcepacks/</span>
                  <span class="text-xs font-mono bg-gray-800/80 text-gray-400 px-2.5 py-1 rounded border border-gray-700/50">options.txt</span>
                  <span class="text-xs font-mono bg-gray-800/80 text-gray-400 px-2.5 py-1 rounded border border-gray-700/50">servers.dat</span>
                </div>
              </div>
            </div>
          </div>

          <!-- Tab: Mods / Resourcepacks -->
          <div v-else class="flex-1 flex flex-col relative rounded-xl border-2 border-dashed transition-all mb-6 overflow-hidden" :class="isDragging ? 'border-green-500 bg-green-500/10' : 'border-gray-800 bg-[#12161d]'">
             
             <!-- Lista de archivos -->
             <div v-if="filesList.length > 0" class="p-5 grid grid-cols-2 gap-3 overflow-y-auto max-h-[300px]">
                <div v-for="file in filesList" :key="file" class="bg-black/50 border border-gray-800 p-3 rounded-lg text-sm text-gray-300 flex items-center gap-3 shadow-sm hover:border-gray-600 transition-colors">
                  <svg class="w-5 h-5 text-blue-500 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z" /></svg>
                  <span class="truncate" :title="file">{{ file }}</span>
                </div>
             </div>

             <!-- Estado vacío -->
             <div v-else class="flex-1 flex flex-col items-center justify-center p-10 text-center">
                <div class="w-16 h-16 bg-gray-900 rounded-full flex items-center justify-center mb-4">
                  <svg class="w-8 h-8 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" /></svg>
                </div>
                <p class="text-gray-300 font-bold text-lg mb-1">Arrastra tus archivos aquí</p>
                <p class="text-gray-500 text-sm">Suelta tus .jar o .zip para instalarlos automáticamente en <span class="font-mono text-gray-400">{{ activeTab }}</span>.</p>
             </div>

             <!-- Overlay arrastre -->
             <div v-if="isDragging" class="absolute inset-0 flex items-center justify-center backdrop-blur-sm z-20 pointer-events-none">
                <div class="bg-green-500 text-black font-extrabold px-6 py-3 rounded-xl shadow-2xl flex items-center gap-2">
                  <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M12 4v16m8-8H4" /></svg>
                  SUELTA PARA INSTALAR
                </div>
             </div>
          </div>

          <!-- Zona de Lanzamiento -->
          <div class="mt-auto w-full max-w-2xl">
            <!-- Barra de progreso -->
            <div v-if="getState(selectedInstance.name).isLaunching" class="mb-4">
              <div class="flex justify-between text-sm text-gray-400 mb-2 font-medium">
                <span>{{ getState(selectedInstance.name).message }}</span>
                <span>{{ getState(selectedInstance.name).progress }}%</span>
              </div>
              <div class="w-full bg-gray-900 rounded-full h-3 shadow-inner overflow-hidden border border-gray-800">
                <div class="bg-green-500 h-full rounded-full transition-all duration-300 relative overflow-hidden" :style="{ width: getState(selectedInstance.name).progress + '%' }">
                  <div class="absolute inset-0 bg-white/20 w-full animate-pulse"></div>
                </div>
              </div>
            </div>

            <!-- Botón Principal -->
            <button
              v-if="getState(selectedInstance.name).isPlaying"
              class="w-full py-5 rounded-xl text-xl font-bold bg-gray-800 text-green-400 border border-green-500/30 cursor-default flex items-center justify-center gap-3 shadow-[0_0_20px_rgba(34,197,94,0.1)]"
            >
              <div class="w-3 h-3 bg-green-400 rounded-full animate-pulse"></div>
              JUEGO EN EJECUCIÓN
            </button>
            <button
              v-else
              @click="lanzar"
              :disabled="getState(selectedInstance.name).isLaunching"
              class="w-full py-5 rounded-xl text-xl font-extrabold bg-green-600 hover:bg-green-500 text-white transition-all disabled:opacity-50 disabled:cursor-not-allowed shadow-[0_0_30px_rgba(22,163,74,0.2)] hover:shadow-[0_0_40px_rgba(22,163,74,0.4)] flex items-center justify-center gap-3"
            >
              <svg v-if="!getState(selectedInstance.name).isLaunching" class="w-6 h-6" fill="currentColor" viewBox="0 0 24 24"><path d="M8 5v14l11-7z"/></svg>
              <svg v-else class="w-6 h-6 animate-spin" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" /></svg>
              {{ getState(selectedInstance.name).isLaunching ? 'CARGANDO...' : 'JUGAR' }}
            </button>
          </div>

        </div>
      </div>
    </main>

    <!-- Modal: Crear instancia -->
    <div v-if="showCreateModal" class="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50" @click.self="showCreateModal = false">
      <div class="bg-[#161b22] border border-gray-700/50 rounded-2xl shadow-2xl w-[420px] p-6 flex flex-col gap-5">
        <h3 class="text-lg font-bold text-white">Nueva Instancia</h3>

        <div class="flex flex-col gap-1">
          <label class="text-xs text-gray-400 font-semibold uppercase tracking-wide">Nombre</label>
          <input v-model="newInstance.name" type="text" placeholder="Mi Survival" class="bg-black border border-gray-700 rounded-lg p-2.5 text-sm focus:outline-none focus:border-blue-500 transition-colors" />
        </div>

        <div class="flex flex-col gap-1">
          <label class="text-xs text-gray-400 font-semibold uppercase tracking-wide">Versión</label>
          <select v-model="newInstance.version_id" :disabled="isLoadingVersions" class="bg-black border border-gray-700 rounded-lg p-2.5 text-sm cursor-pointer focus:outline-none focus:border-blue-500">
            <option v-if="isLoadingVersions" value="">Cargando versiones...</option>
            <option v-for="ver in availableVersions" :key="ver" :value="ver">{{ ver }}</option>
          </select>
        </div>

        <div class="flex gap-3">
          <div class="flex-1 flex flex-col gap-1">
            <label class="text-xs text-gray-400 font-semibold uppercase tracking-wide">Usuario</label>
            <input v-model="newInstance.username" type="text" placeholder="NeoDev" class="bg-black border border-gray-700 rounded-lg p-2.5 text-sm focus:outline-none focus:border-blue-500 transition-colors" />
          </div>
          <div class="w-28 flex flex-col gap-1">
            <label class="text-xs text-gray-400 font-semibold uppercase tracking-wide">RAM</label>
            <select v-model="newInstance.max_memory" class="bg-black border border-gray-700 rounded-lg p-2.5 text-sm cursor-pointer focus:outline-none focus:border-blue-500">
              <option value="2G">2 GB</option>
              <option value="4G">4 GB</option>
              <option value="6G">6 GB</option>
              <option value="8G">8 GB</option>
            </select>
          </div>
        </div>

        <div class="flex gap-3 mt-2">
          <button @click="showCreateModal = false" class="flex-1 py-2.5 rounded-xl text-sm font-semibold border border-gray-700 text-gray-400 hover:text-white hover:border-gray-500 transition-all">
            Cancelar
          </button>
          <button @click="crearInstancia" :disabled="!newInstance.name || !newInstance.version_id" class="flex-1 py-2.5 rounded-xl text-sm font-bold bg-green-600 hover:bg-green-500 text-white transition-all disabled:opacity-40 disabled:cursor-not-allowed">
            Crear
          </button>
        </div>
      </div>
    </div>

  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { ask, message } from '@tauri-apps/plugin-dialog';
import { appDataDir } from '@tauri-apps/api/path';

interface Instance {
  name: string;
  version_id: string;
  created_at: string;
  last_played: string | null;
  max_memory: string;
  username: string;
}

const instances = ref<Instance[]>([]);
const selectedInstance = ref<Instance | null>(null);
const availableVersions = ref<string[]>([]);
const isLoadingVersions = ref(true);
const showCreateModal = ref(false);
const baseDataDir = ref('~/.isocraft');

const activeTab = ref('general');
const filesList = ref<string[]>([]);
const isDragging = ref(false);

async function loadFiles() {
  if (!selectedInstance.value || activeTab.value === 'general') return;
  try {
    filesList.value = await invoke<string[]>('listar_archivos', {
      instanceName: selectedInstance.value.name,
      folder: activeTab.value
    });
  } catch (error) {
    console.error("Error listando archivos:", error);
  }
}

watch(activeTab, loadFiles);
watch(selectedInstance, () => {
  if (activeTab.value !== 'general') loadFiles();
});

interface InstanceState {
  isLaunching: boolean;
  isPlaying: boolean;
  progress: number;
  message: string;
}
const states = ref<Record<string, InstanceState>>({});

function getState(name: string): InstanceState {
  if (!states.value[name]) {
    states.value[name] = { isLaunching: false, isPlaying: false, progress: 0, message: '' };
  }
  return states.value[name];
}

const newInstance = ref({
  name: '',
  version_id: '',
  username: 'NeoDev',
  max_memory: '4G',
});

onMounted(async () => {
  // Obtener ruta real
  try {
    const dir = await appDataDir();
    baseDataDir.value = dir.replace(/[\/\\]$/, '');
  } catch (e) {
    console.warn("No se pudo obtener appDataDir", e);
  }

  // Escuchar eventos de progreso
  listen<{ instance_name: string, progress: number, message: string }>('jre-progress', (event) => {
    const st = getState(event.payload.instance_name);
    st.progress = event.payload.progress;
    st.message = event.payload.message;
  });

  listen<{ instance_name: string, info: string }>('game-launched', (event) => {
    const st = getState(event.payload.instance_name);
    st.isLaunching = false;
    st.isPlaying = true;
  });

  listen<string>('game-closed', (event) => {
    const st = getState(event.payload);
    st.isPlaying = false;
  });

  // Listen for file drops
  listen<{ paths: string[] }>('tauri://drag-drop', async (event) => {
    isDragging.value = false;
    if (activeTab.value === 'general' || !selectedInstance.value) return;
    
    let imported = 0;
    for (const p of event.payload.paths) {
      if (p.endsWith('.jar') || p.endsWith('.zip')) {
        try {
          await invoke('importar_archivo', {
            instanceName: selectedInstance.value.name,
            folder: activeTab.value,
            sourcePath: p
          });
          imported++;
        } catch(e) {
          console.error(e);
        }
      }
    }
    
    if (imported > 0) {
      loadFiles();
      await message(`Se instalaron ${imported} archivos correctamente en la carpeta ${activeTab.value}.`, { title: 'Instalación Exitosa', kind: 'info' });
    }
  });

  listen('tauri://drag-enter', () => {
    if (activeTab.value !== 'general') isDragging.value = true;
  });
  
  listen('tauri://drag-leave', () => {
    isDragging.value = false;
  });

  // Cargar instancias existentes
  try {
    instances.value = await invoke<Instance[]>('listar_instancias');
    if (instances.value.length > 0) {
      selectedInstance.value = instances.value[0];
    }
  } catch (error) {
    console.error("Error cargando instancias:", error);
  }

  // Cargar versiones disponibles
  try {
    const versiones = await invoke<string[]>('obtener_versiones');
    availableVersions.value = versiones;
    if (versiones.length > 0) {
      newInstance.value.version_id = versiones[0];
    }
  } catch (error) {
    console.error("No se pudo obtener el manifiesto:", error);
  } finally {
    isLoadingVersions.value = false;
  }
});

function selectInstance(inst: Instance) {
  selectedInstance.value = inst;
}

async function crearInstancia() {
  try {
    const created = await invoke<Instance>('crear_instancia', {
      name: newInstance.value.name,
      versionId: newInstance.value.version_id,
      username: newInstance.value.username,
      maxMemory: newInstance.value.max_memory,
    });
    instances.value.unshift(created);
    selectedInstance.value = created;
    showCreateModal.value = false;
    newInstance.value = { name: '', version_id: availableVersions.value[0] || '', username: 'NeoDev', max_memory: '4G' };
  } catch (error) {
    console.error("Error creando instancia:", error);
    await message(String(error), { title: 'Error al crear', kind: 'error' });
  }
}

async function lanzar() {
  if (!selectedInstance.value) return;
  const name = selectedInstance.value.name;

  const st = getState(name);
  if (st.isLaunching || st.isPlaying) return;

  st.isLaunching = true;
  st.progress = 0;
  st.message = "Consultando Manifiesto...";

  try {
    await invoke('iniciar_pipeline_dinamico', {
      instanceName: name,
    });
  } catch (error) {
    console.error(error);
    await message(String(error), { title: 'Error de Lanzamiento', kind: 'error' });
  } finally {
    // Si no falló, isLaunching se hace false en el evento game-launched
    // Pero si falló, lo reseteamos aquí. Si no falló, game-launched debería manejarlo.
    // Sin embargo, por simplicidad, lo podemos dejar en false al final si falló.
    // Usaremos un check por si no está en isPlaying:
    if (!st.isPlaying) {
       st.isLaunching = false;
    }
    // Recargar instancias para actualizar last_played
    instances.value = await invoke<Instance[]>('listar_instancias');
  }
}

async function abrirCarpeta(name: string) {
  try {
    await invoke('abrir_carpeta_instancia', { name });
  } catch (error) {
    console.error(error);
    await message(String(error), { title: 'Error al abrir', kind: 'error' });
  }
}

async function confirmDelete() {
  if (!selectedInstance.value) return;
  const name = selectedInstance.value.name;
  
  const st = getState(name);
  if (st.isLaunching || st.isPlaying) {
    // Forzamos el backend a matarlo
    const yes = await ask(`La instancia "${name}" está en ejecución. ¿Forzar el cierre y ELIMINARLA por completo?`, { title: 'Confirmar Eliminación', kind: 'warning' });
    if (!yes) return;
  } else {
    const yes = await ask(`¿Eliminar la instancia "${name}" y todos sus datos (saves, mods, configs)?`, { title: 'Confirmar Eliminación', kind: 'warning' });
    if (!yes) return;
  }

  try {
    await invoke('eliminar_instancia', { name });
    instances.value = instances.value.filter(i => i.name !== name);
    selectedInstance.value = instances.value.length > 0 ? instances.value[0] : null;
    st.isPlaying = false;
    st.isLaunching = false;
  } catch (error) {
    console.error(error);
    await message(String(error), { title: 'Error al eliminar', kind: 'error' });
  }
}
</script>
