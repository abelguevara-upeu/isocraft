import { invoke } from "@tauri-apps/api/core";

/**
 * Centralized API service for interacting with the Rust backend.
 * This avoids hardcoding command names in the UI components.
 */

export interface LoaderVersionMapping {
  minecraft: string;
  loader: string;
}

export interface InstanceConfig {
  name: string;
  version_id: string;
  loader: string;
  loader_version: string;
  created_at: string;
  last_played: string | null;
  max_memory: string;
  username: string;
}

export const InstanceService = {
  async getVersions(loader: string): Promise<LoaderVersionMapping[]> {
    return await invoke("get_versions", { loader });
  },

  async listInstances(): Promise<InstanceConfig[]> {
    return await invoke("list_instances");
  },

  async createInstance(config: Partial<InstanceConfig>): Promise<InstanceConfig> {
    // We map snake_case to camelCase for the Tauri invoke call
    return await invoke("create_instance", {
      name: config.name,
      versionId: config.version_id,
      loader: config.loader,
      loaderVersion: config.loader_version,
      username: config.username,
      maxMemory: config.max_memory
    });
  },

  async deleteInstance(name: string): Promise<void> {
    return await invoke("delete_instance", { name });
  },

  async openFolder(name: string): Promise<void> {
    return await invoke("open_instance_folder", { name });
  }
};

export const LauncherService = {
  async launchInstance(instanceName: string): Promise<void> {
    return await invoke("launch_instance", { instanceName });
  }
};

export interface ImportResult {
  imported: number;
  skipped: number;
}

export const FileService = {
  async listFiles(instanceName: string, folder: string): Promise<string[]> {
    return await invoke("list_files", { instanceName, folder });
  },

  /** Accepts a file path OR a folder path. Returns how many were imported vs skipped (duplicates). */
  async importPath(instanceName: string, folder: string, sourcePath: string): Promise<ImportResult> {
    return await invoke<ImportResult>("import_path", { instanceName, folder, sourcePath });
  }
};
