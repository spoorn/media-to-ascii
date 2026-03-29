<script setup lang="ts">
import { ref, provide } from "vue";
import { invoke } from "@tauri-apps/api/core";
import Video from "./video/Video.vue";
import ProgressBar from 'primevue/progressbar';
import Button from 'primevue/button';

interface VideoProgress {
  percentage: number;
  currentReadFrame: number;
  currentEncodeFrame: number;
  currentWriteFrame: number;
  totalFrames: number;
}

const processing = ref(false);
const progress = ref<VideoProgress>({ percentage: 0, currentReadFrame: 0, currentEncodeFrame: 0, currentWriteFrame: 0, totalFrames: 0 });

const elapsedTime = ref(0);
let timerInterval: number | null = null;

function startTimer() {
    const startTime = Date.now();
    timerInterval = window.setInterval(() => {
        elapsedTime.value = Math.floor((Date.now() - startTime) / 1000);
    }, 1000);
}

function stopTimer() {
    if (timerInterval) {
        clearInterval(timerInterval);
        timerInterval = null;
    }
    elapsedTime.value = 0;
}

function formatTime(seconds: number): string {
    const hrs = Math.floor(seconds / 3600);
    const mins = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    return `${hrs.toString().padStart(2, '0')}:${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
}

provide('processing', processing);
provide('progress', progress);
provide('startTimer', startTimer);
provide('stopTimer', stopTimer);

async function cancelProcessing() {
    await invoke('cancel_processing');
}
</script>

<template>
  <div class="app-container">
    <div class="settings mx-auto">
      <Video/>
    </div>
    <div v-if="processing" class="progress-container">
      <div class="progress-content">
        <div class="flex-1">
          <ProgressBar :value="progress.percentage" class="h-3" />
          <div class="grid grid-cols-4 gap-4 text-sm text-gray-500">
            <div>
              <p>
                Read Frame {{ progress.currentReadFrame }} of {{ progress.totalFrames }}
              </p>
            </div>

            <div>
              <p>
                Encode Frame {{ progress.currentEncodeFrame }} of {{ progress.totalFrames }}
              </p>
            </div>

            <div>
              <p>
                Write Frame {{ progress.currentWriteFrame }} of {{ progress.totalFrames }}
              </p>
            </div>

            <div class="text-right">
              <p>
                Elapsed {{ formatTime(elapsedTime) }}
              </p>
            </div>
          </div>
        </div>
        <Button
          label="Cancel"
          severity="danger"
          size="small"
          @click="cancelProcessing"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
</style>
<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.app-container {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
  height: 100%;
}

.settings {
  flex: 1;
  margin: 0;
  padding: 0.5rem;
  display: flex;
  flex-direction: column;
  text-align: center;
  width: 100%;
  overflow-y: auto;
}

.progress-container {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  padding: 0.75rem 1rem;
  background: #f6f6f6;
  border-top: 1px solid #ccc;
  z-index: 100;
}

.progress-content {
  display: flex;
  align-items: center;
  gap: 1rem;
  width: 100%;
}

a {
  font-weight: 500;
  color: #646cff;
  text-decoration: inherit;
}

a:hover {
  color: #535bf2;
}

h1 {
  text-align: center;
}

input,
button {
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
}

button {
  cursor: pointer;
}

button:hover {
  border-color: #d94a8c;
}

button:active {
  border-color: #d94a8c;
  background-color: #ffd6eb;
}

input,
button {
  outline: none;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  a:hover {
    color: #24c8db;
  }

  input,
  button {
    color: #ffffff;
    background-color: #0f0f0f98;
  }

button:active {
  background-color: #0f0f0f69;
}

.p-progressbar .p-progressbar-value {
  background: #ffb8db;
}
}
</style>
