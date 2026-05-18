<template>
  <div class="fixed inset-0 bg-black/80 backdrop-blur-md flex items-center justify-center z-[100] animate-fade-in" @click.self="$emit('close')">
    <div class="premium-card w-[480px] p-8 flex flex-col gap-6 shadow-2xl relative overflow-hidden">
      <!-- Decoration -->
      <div class="absolute -top-24 -right-24 w-48 h-48 bg-purple-500/10 rounded-full blur-3xl"></div>
      
      <header>
        <h3 class="text-2xl font-black text-white tracking-tight">Edit Instance</h3>
        <p class="text-sm text-gray-500 font-medium">Modify your isolated environment settings</p>
      </header>

      <div class="space-y-4">
        <div class="flex flex-col gap-2">
          <label class="text-[10px] font-black uppercase tracking-widest text-gray-400">Instance Name</label>
          <input v-model="form.name" type="text" placeholder="My Epic Survival" class="input-field" />
        </div>

        <div class="grid grid-cols-2 gap-4">
          <div class="flex flex-col gap-2">
            <label class="text-[10px] font-black uppercase tracking-widest text-gray-400">Mod Loader</label>
            <select v-model="form.loader" class="input-field cursor-pointer appearance-none">
              <option value="Vanilla">Vanilla</option>
              <option value="Fabric">Fabric</option>
              <option value="Forge">Forge</option>
              <option value="NeoForge">NeoForge</option>
            </select>
          </div>

          <div class="flex flex-col gap-2">
            <label class="text-[10px] font-black uppercase tracking-widest text-gray-400">Version</label>
            <div class="relative">
              <select v-model="selectedMappingIndex" :disabled="loadingVersions" class="input-field w-full cursor-pointer appearance-none disabled:opacity-50">
                <option v-if="loadingVersions" value="-1">Loading...</option>
                <option v-for="(mapping, index) in versions" :key="mapping.minecraft" :value="index">
                  {{ mapping.minecraft }} {{ form.loader !== 'Vanilla' ? '(' + mapping.loader + ')' : '' }}
                </option>
              </select>
              <div v-if="loadingVersions" class="absolute right-3 top-3.5">
                <div class="w-4 h-4 border-2 border-purple-500 border-t-transparent rounded-full animate-spin"></div>
              </div>
            </div>
          </div>
        </div>

        <div class="grid grid-cols-2 gap-4">
          <div class="flex flex-col gap-2">
            <label class="text-[10px] font-black uppercase tracking-widest text-gray-400">Username</label>
            <input v-model="form.username" type="text" placeholder="Player" class="input-field" />
          </div>
          <div class="flex flex-col gap-2">
            <label class="text-[10px] font-black uppercase tracking-widest text-gray-400">Max RAM</label>
            <select v-model="form.max_memory" class="input-field cursor-pointer appearance-none">
              <option value="2G">2 GB</option>
              <option value="4G">4 GB</option>
              <option value="6G">6 GB</option>
              <option value="8G">8 GB</option>
              <option value="12G">12 GB</option>
            </select>
          </div>
        </div>
      </div>

      <div class="flex gap-3 mt-4">
        <button @click="$emit('close')" class="flex-1 btn-secondary">
          Cancel
        </button>
        <button 
          @click="handleSave" 
          :disabled="!form.name || selectedMappingIndex === -1 || saving" 
          class="flex-1 btn-primary bg-purple-600 hover:bg-purple-500 shadow-purple-900/30 hover:shadow-purple-500/20 disabled:opacity-40 disabled:active:scale-100 flex items-center justify-center gap-2"
        >
          <span v-if="saving" class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></span>
          {{ saving ? 'Saving...' : 'Save Changes' }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import type { InstanceConfig, LoaderVersionMapping } from '../services/api';

const props = defineProps<{
  instance: InstanceConfig;
  versions: LoaderVersionMapping[];
  loadingVersions: boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'save', data: any): void;
  (e: 'change-loader', loader: string): void;
}>();

const saving = ref(false);
const selectedMappingIndex = ref(-1);

const form = ref({
  name: props.instance.name,
  version_id: props.instance.version_id,
  loader: props.instance.loader,
  loader_version: props.instance.loader_version,
  username: props.instance.username,
  max_memory: props.instance.max_memory,
});

watch(() => form.value.loader, (newLoader) => {
  selectedMappingIndex.value = -1;
  form.value.version_id = '';
  form.value.loader_version = '';
  emit('change-loader', newLoader);
});

onMounted(() => {
  emit('change-loader', form.value.loader);
});

watch(() => props.versions, (newVersions) => {
  if (newVersions.length > 0) {
    // If name and loader match props.instance, attempt to select props.instance version first
    const isCurrentLoader = form.value.loader === props.instance.loader;
    const targetVersion = isCurrentLoader ? props.instance.version_id : null;
    const targetLoaderVersion = isCurrentLoader ? props.instance.loader_version : null;

    let idx = -1;
    if (targetVersion) {
      idx = newVersions.findIndex(
        (v) => v.minecraft === targetVersion && (!targetLoaderVersion || v.loader === targetLoaderVersion)
      );
    }

    if (idx !== -1) {
      selectedMappingIndex.value = idx;
    } else {
      selectedMappingIndex.value = 0;
      form.value.version_id = newVersions[0].minecraft;
      form.value.loader_version = newVersions[0].loader;
    }
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

async function handleSave() {
  saving.value = true;
  try {
    await emit('save', { ...form.value });
  } finally {
    saving.value = false;
  }
}
</script>
