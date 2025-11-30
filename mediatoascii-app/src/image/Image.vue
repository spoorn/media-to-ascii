<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { openPath } from "@tauri-apps/plugin-opener";
import { invoke } from "@tauri-apps/api/core";
import { defaultImageConfig, type ImageConfig } from "./image";

const STORAGE_KEY = "mediatoascii:image-config";

const config = ref<ImageConfig>(loadPersistedConfig());
const status = ref<"idle" | "processing" | "done" | "error">("idle");
const statusDetail = ref("Ready to process an image");
const processError = ref<string | null>(null);
const processing = ref(false);
const outputs = ref<string[]>([]);

function loadPersistedConfig(): ImageConfig {
  const saved = localStorage.getItem(STORAGE_KEY);
  if (saved) {
    try {
      return { ...defaultImageConfig(), ...JSON.parse(saved) };
    } catch (_) {
      return defaultImageConfig();
    }
  }
  return defaultImageConfig();
}

watch(
  config,
  (value) => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(value));
  },
  { deep: true }
);

const validationErrors = computed(() => {
  const errors: string[] = [];
  if (!config.value.image_path.trim()) {
    errors.push("Input image is required.");
  }
  if (!config.value.output_file_path && !config.value.output_image_path) {
    errors.push("Choose at least one output: text file or image file.");
  }
  if (config.value.scale_down <= 0) {
    errors.push("Scale down must be greater than 0.");
  }
  if (config.value.font_size <= 0) {
    errors.push("Font size must be greater than 0.");
  }
  if (config.value.height_sample_scale <= 0) {
    errors.push("Height sample scale must be greater than 0.");
  }
  return errors;
});

const canSubmit = computed(
  () => !processing.value && validationErrors.value.length === 0
);

async function pickImage() {
  const selected = await openDialog({
    multiple: false,
    filters: [
      {
        name: "Images",
        extensions: ["png", "jpg", "jpeg", "gif", "bmp", "webp"],
      },
    ],
  });
  if (typeof selected === "string") {
    config.value.image_path = selected;
  }
}

async function pickTextOutput() {
  const selected = await openDialog({
    directory: true,
    multiple: false,
    defaultPath: config.value.output_file_path
      ? config.value.output_file_path.substring(
          0,
          config.value.output_file_path.lastIndexOf("/")
        )
      : undefined,
  });
  if (typeof selected === "string") {
    const separator = selected.includes("\\") ? "\\" : "/";
    const path = selected.endsWith(separator) ? selected : selected + separator;
    config.value.output_file_path = path + "output.txt";
  }
}

async function pickImageOutput() {
  const selected = await openDialog({
    directory: true,
    multiple: false,
    defaultPath: config.value.output_image_path
      ? config.value.output_image_path.substring(
          0,
          config.value.output_image_path.lastIndexOf("/")
        )
      : undefined,
  });
  if (typeof selected === "string") {
    const separator = selected.includes("\\") ? "\\" : "/";
    const path = selected.endsWith(separator) ? selected : selected + separator;
    config.value.output_image_path = path + "output.png";
  }
}

function resetStatus() {
  status.value = "idle";
  statusDetail.value = "Ready to process an image";
  processError.value = null;
  outputs.value = [];
}

async function processImage() {
  processError.value = null;
  status.value = "processing";
  statusDetail.value = "Converting image...";
  outputs.value = [];

  const payload: ImageConfig = {
    ...config.value,
    output_file_path: config.value.output_file_path?.trim() || null,
    output_image_path: config.value.output_image_path?.trim() || null,
  };

  processing.value = true;
  try {
    await invoke("process_image", { config: payload });
    status.value = "done";
    statusDetail.value = "ASCII image ready";
    outputs.value = [
      payload.output_file_path,
      payload.output_image_path,
    ].filter(Boolean) as string[];
  } catch (error: any) {
    processError.value = error?.message || String(error);
    status.value = "error";
    statusDetail.value = "Something went wrong";
  } finally {
    processing.value = false;
  }
}

async function openOutput(path: string) {
  try {
    await openPath(path);
  } catch (err) {
    console.error("Could not open output file", err);
  }
}
</script>

<template>
  <div class="space-y-8 animate-in fade-in duration-500">
    <!-- Header Section -->
    <div class="flex flex-col gap-2">
      <h2 class="text-2xl font-bold text-white tracking-tight neon-text">
        Image Processing
      </h2>
      <p class="text-slate-400">
        Convert static imagery into detailed ASCII representations.
      </p>
    </div>

    <div class="grid gap-6 lg:grid-cols-2">
      <!-- Input/Output Section -->
      <div class="glass-panel rounded-2xl p-6 space-y-6">
        <div class="flex items-center gap-2 mb-4">
          <div
            class="h-1 w-8 bg-cyan-500 rounded-full shadow-[0_0_10px_rgba(6,182,212,0.5)]"
          ></div>
          <h3 class="text-sm font-bold uppercase tracking-widest text-cyan-400">
            Source & Destination
          </h3>
        </div>

        <!-- Input -->
        <div class="space-y-2">
          <div class="flex justify-between items-center">
            <label
              class="text-xs font-semibold text-slate-300 uppercase tracking-wider"
              >Input Image</label
            >
            <button
              type="button"
              class="text-xs text-cyan-400 hover:text-cyan-300 transition-colors"
              @click="pickImage"
              :disabled="processing"
            >
              BROWSE FILES
            </button>
          </div>
          <div class="relative group">
            <input
              v-model="config.image_path"
              :disabled="processing"
              placeholder="Select image..."
              class="glass-input w-full rounded-lg px-4 py-3 text-sm placeholder-slate-500"
            />
            <div
              class="absolute inset-0 rounded-lg ring-1 ring-white/10 pointer-events-none group-hover:ring-white/20 transition-all"
            ></div>
          </div>
        </div>

        <!-- Outputs -->
        <div class="space-y-4">
          <!-- Text Output -->
          <div class="space-y-2">
            <div class="flex justify-between items-center">
              <label
                class="text-xs font-semibold text-slate-300 uppercase tracking-wider"
                >Text Output (Optional)</label
              >
              <button
                type="button"
                class="text-xs text-cyan-400 hover:text-cyan-300 transition-colors"
                @click="pickTextOutput"
                :disabled="processing"
              >
                CHOOSE FOLDER
              </button>
            </div>
            <div class="relative group">
              <input
                v-model="config.output_file_path"
                :disabled="processing"
                placeholder="/path/to/output.txt"
                class="glass-input w-full rounded-lg px-4 py-3 text-sm placeholder-slate-500"
              />
              <div
                class="absolute inset-0 rounded-lg ring-1 ring-white/10 pointer-events-none group-hover:ring-white/20 transition-all"
              ></div>
            </div>
          </div>

          <!-- Image Output -->
          <div class="space-y-2">
            <div class="flex justify-between items-center">
              <label
                class="text-xs font-semibold text-slate-300 uppercase tracking-wider"
                >Image Output (Optional)</label
              >
              <button
                type="button"
                class="text-xs text-cyan-400 hover:text-cyan-300 transition-colors"
                @click="pickImageOutput"
                :disabled="processing"
              >
                CHOOSE FOLDER
              </button>
            </div>
            <div class="relative group">
              <input
                v-model="config.output_image_path"
                :disabled="processing"
                placeholder="/path/to/output.png"
                class="glass-input w-full rounded-lg px-4 py-3 text-sm placeholder-slate-500"
              />
              <div
                class="absolute inset-0 rounded-lg ring-1 ring-white/10 pointer-events-none group-hover:ring-white/20 transition-all"
              ></div>
            </div>
          </div>
        </div>

        <!-- Options -->
        <div class="flex flex-wrap gap-4 pt-2">
          <label class="flex items-center gap-3 cursor-pointer group">
            <div class="relative flex items-center">
              <input
                type="checkbox"
                v-model="config.overwrite"
                :disabled="processing"
                class="peer sr-only"
              />
              <div
                class="w-9 h-5 bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-cyan-600"
              ></div>
            </div>
            <span
              class="text-sm text-slate-300 group-hover:text-white transition-colors"
              >Overwrite existing</span
            >
          </label>
        </div>
      </div>

      <!-- Settings Section -->
      <div class="glass-panel rounded-2xl p-6 space-y-6">
        <div class="flex items-center gap-2 mb-4">
          <div
            class="h-1 w-8 bg-purple-500 rounded-full shadow-[0_0_10px_rgba(168,85,247,0.5)]"
          ></div>
          <h3
            class="text-sm font-bold uppercase tracking-widest text-purple-400"
          >
            Configuration
          </h3>
        </div>

        <div class="grid grid-cols-2 gap-4">
          <label class="space-y-2">
            <span class="text-xs font-semibold text-slate-400">Scale Down</span>
            <input
              type="number"
              step="0.1"
              min="0.1"
              v-model.number="config.scale_down"
              :disabled="processing"
              class="glass-input w-full rounded-lg px-3 py-2 text-sm"
            />
          </label>
          <label class="space-y-2">
            <span class="text-xs font-semibold text-slate-400">Font Size</span>
            <input
              type="number"
              step="0.5"
              min="1"
              v-model.number="config.font_size"
              :disabled="processing"
              class="glass-input w-full rounded-lg px-3 py-2 text-sm"
            />
          </label>
          <label class="space-y-2 col-span-2">
            <span class="text-xs font-semibold text-slate-400"
              >Height Sample Scale</span
            >
            <input
              type="number"
              step="0.01"
              min="0.1"
              v-model.number="config.height_sample_scale"
              :disabled="processing"
              class="glass-input w-full rounded-lg px-3 py-2 text-sm"
            />
          </label>
        </div>

        <label class="flex items-center gap-3 cursor-pointer group pt-2">
          <div class="relative flex items-center">
            <input
              type="checkbox"
              v-model="config.invert"
              :disabled="processing"
              class="peer sr-only"
            />
            <div
              class="w-9 h-5 bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-purple-600"
            ></div>
          </div>
          <span
            class="text-sm text-slate-300 group-hover:text-white transition-colors"
            >Invert Colors</span
          >
        </label>
      </div>
    </div>

    <!-- Status & Actions -->
    <div class="glass-panel rounded-2xl p-6 border-t-2 border-t-cyan-500/20">
      <div class="flex flex-col md:flex-row gap-6 items-center justify-between">
        <div class="w-full md:w-2/3 space-y-3">
          <div class="flex justify-between items-end">
            <div>
              <h4 class="text-sm font-bold text-white uppercase tracking-wider">
                {{
                  status === "processing"
                    ? "RENDERING..."
                    : status === "done"
                    ? "COMPLETE"
                    : status === "error"
                    ? "FAILED"
                    : "READY"
                }}
              </h4>
              <p class="text-xs text-slate-400 mt-1 font-mono">
                {{ statusDetail }}
              </p>
            </div>
          </div>

          <div
            v-if="processError"
            class="text-xs text-red-400 bg-red-900/20 border border-red-900/50 p-2 rounded"
          >
            {{ processError }}
          </div>

          <div v-if="outputs.length" class="space-y-2">
            <div
              v-for="out in outputs"
              :key="out"
              class="flex items-center gap-3 text-xs bg-emerald-900/20 border border-emerald-900/50 p-2 rounded text-emerald-300"
            >
              <span class="font-semibold">SAVED:</span>
              <span class="truncate flex-1 font-mono opacity-80">{{
                out
              }}</span>
              <button
                @click="openOutput(out)"
                class="hover:text-white underline decoration-emerald-500/50 hover:decoration-emerald-500"
              >
                OPEN
              </button>
            </div>
          </div>

          <div
            v-if="validationErrors.length"
            class="text-xs text-amber-400 bg-amber-900/20 border border-amber-900/50 p-2 rounded"
          >
            <ul class="list-disc pl-4 space-y-0.5">
              <li v-for="err in validationErrors" :key="err">{{ err }}</li>
            </ul>
          </div>
        </div>

        <div class="flex gap-3 w-full md:w-auto">
          <button
            type="button"
            class="btn-secondary px-6 py-3 rounded-lg font-bold text-sm tracking-wide"
            @click="resetStatus"
            :disabled="processing"
          >
            RESET
          </button>
          <button
            type="button"
            class="btn-primary flex-1 md:flex-none px-8 py-3 rounded-lg shadow-lg shadow-cyan-500/20 disabled:opacity-50 disabled:cursor-not-allowed"
            :disabled="!canSubmit"
            @click="processImage"
          >
            {{ processing ? "PROCESSING..." : "INITIATE" }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
