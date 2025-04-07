<script setup lang="ts">
import { ref } from "vue";
import { defaultVideoConfig } from "./video.ts";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import ProgressBar from 'primevue/progressbar';

const config = ref(defaultVideoConfig());
const processError = ref(null);
const processing = ref(false);
const progress = ref(0.0);

async function processVideo() {
  processError.value = null;
  processing.value = true;
  invoke('video_progress')
      .then(() => {
        console.log('Video progress done');
      });
  invoke('process_video', {config: config.value})
      .then(() => {
        console.log('Video processing done');
        processing.value = false;
        progress.value = 1.0;
      })
      .catch((error) => processError.value = error);
}

listen<number>('video-progress', (event) => {
  progress.value = Math.floor(event.payload * 100);
});
</script>

<template>
  <div>
    <h1>Video Settings</h1>

    <form @submit.prevent="processVideo">
      <div>
        <label for="video-input">Input Video Path</label>
        <input id="video-input" :disabled="processing == true" v-model="config.video_path"
               placeholder="input video path..."/>
      </div>
      <div>
        <label for="scale-down">Scale Down</label>
        <input id="scale-down" type="number" step="any" v-model.number="config.scale_down"/>
      </div>
      <div>
        <label for="overwrite">Overwrite</label>
        <input type="checkbox" v-model="config.overwrite" id="overwrite"/>
      </div>
      <div>
        <label for="video-width">Width</label>
        <input id="video-output" v-model="config.output_video_path" placeholder="output path..."/>
      </div>
      <div>
        <button type="submit" :disabled="processing == true">Asciify</button>
      </div>
      <div v-if="processError" class="text-red-500">{{ processError }}</div>
      <ProgressBar :value="progress" class="m-2"></ProgressBar>
    </form>
  </div>
</template>

<style scoped>

</style>