<script setup lang="ts">
import { ref, inject, type Ref } from "vue";
import { defaultVideoConfig, rotateOptions } from "./video.ts";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open, save } from "@tauri-apps/plugin-dialog";
import InputText from 'primevue/inputtext';
import InputNumber from 'primevue/inputnumber';
import Select from 'primevue/select';
import Checkbox from 'primevue/checkbox';
import ToggleButton from 'primevue/togglebutton';
import Button from 'primevue/button';

interface VideoProgress {
    percentage: number;
    currentReadFrame: number;
    currentEncodeFrame: number;
    currentWriteFrame: number;
    totalFrames: number;
}

const config = ref(defaultVideoConfig());
const processError = ref<string | null>(null);
const anyProcessed = inject<Ref<boolean>>('anyProcessed', ref(false));
const processing = inject<Ref<boolean>>('processing', ref(false));
const progress = inject<Ref<VideoProgress>>('progress', ref({ percentage: 0, currentReadFrame: 0, currentEncodeFrame: 0, currentWriteFrame: 0, totalFrames: 0 }));
const startTimer = inject<() => void>('startTimer');
const stopTimer = inject<() => void>('stopTimer');
const selectedRotate = ref(-1);

async function browseInputVideo() {
    const selected = await open({
        multiple: false,
        filters: [{
            name: 'Video',
            extensions: ['mp4', 'avi', 'mov', 'mkv', 'webm', 'wmv', 'flv', 'm4v']
        }]
    });
    if (selected) {
        config.value.video_path = selected as string;
    }
}

async function browseOutputVideo() {
    const selected = await save({
        filters: [{
            name: 'Video',
            extensions: ['mp4']
        }],
        defaultPath: 'ascii_output.mp4',
    });
    if (selected) {
        // const exists = await invoke<boolean>('file_exists', { path: selected });
        // if (exists) {
        //     const overwrite = await confirm(
        //         'File already exists. Do you want to overwrite it?',
        //         { title: 'Confirm Overwrite', kind: 'warning' }
        //     );
        //     if (!overwrite) {
        //         return;
        //     }
        // }
        config.value.output_video_path = selected;
    }
}

async function processVideo() {
    config.value.rotate = selectedRotate.value;
    processError.value = null;
    processing.value = true;
    anyProcessed.value = true;
    progress.value = { percentage: 0, currentReadFrame: 0, currentEncodeFrame: 0, currentWriteFrame: 0, totalFrames: 0 };
    startTimer?.();

    invoke('video_progress')
        .then(() => {
            console.log('Video progress done');
        });

    invoke('process_video', { config: config.value })
        .then(() => {
            console.log('Video processing done');
            // Artificially set progress to 100% on completion in case final event(s) were missed
            progress.value.percentage = 100;
            progress.value.currentWriteFrame = progress.value.totalFrames;
        })
        .catch((error) => {
            processError.value = error as string;
        })
        .finally(() => {
            processing.value = false;
            stopTimer?.();
        });
}

listen<{ percentage: number; current_read_frame: number; current_encode_frame: number; current_write_frame: number; total_frames: number }>('video-progress', (event) => {
    progress.value = {
        percentage: Math.floor(event.payload.percentage * 100),
        currentReadFrame: event.payload.current_read_frame,
        currentEncodeFrame: event.payload.current_encode_frame,
        currentWriteFrame: event.payload.current_write_frame,
        totalFrames: event.payload.total_frames,
    };
});
</script>

<template>
    <div class="video-settings">
        <h1 class="text-xl font-bold mb-3 text-center">Video to ASCII Converter</h1>

        <form @submit.prevent="processVideo">
            <div class="settings-grid">
                <div class="settings-column">
                    <h2 class="text-base font-semibold mb-2 border-b border-gray-600 pb-1">Input/Ascii Settings</h2>

                    <div class="form-group">
                        <label for="video-input" class="block mb-0.5 text-sm font-medium">Input Video Path</label>
                        <small class="text-gray-500">Required input video file path</small>
                        <div class="flex gap-2">
                            <InputText
                                id="video-input"
                                v-model="config.video_path"
                                placeholder="Select a video file..."
                                class="flex-1"
                                :disabled="processing"
                            />
                            <Button
                                type="button"
                                label="Browse"
                                :disabled="processing"
                                @click="browseInputVideo"
                            />
                        </div>
                    </div>

                    <div class="form-group">
                        <label for="scale-down" class="block mb-0.5 text-sm font-medium">Scale Down</label>
                        <small class="text-gray-500 text-left">
                            Multiplier to scale down input dimensions. For the output codec, you'll be required to scale
                            down the video to a supported resolution. Recommend 4.0 or higher for 1080p inputs
                        </small>
                        <InputNumber
                            id="scale-down"
                            v-model="config.scale_down"
                            :min="0.1"
                            :max="100"
                            :step="1"
                            :minFractionDigits="1"
                            :maxFractionDigits="2"
                            :showButtons="true"
                            :disabled="processing"
                        />
                    </div>

                    <div class="form-group">
                        <label for="font-size" class="block mb-0.5 text-sm font-medium">Font Size</label>
                        <small class="text-gray-500">Affects output quality and resolution</small>
                        <InputNumber
                            id="font-size"
                            v-model="config.font_size"
                            :min="6"
                            :max="72"
                            :step="1"
                            :showButtons="true"
                            :disabled="processing"
                        />
                    </div>

                    <div class="form-group">
                        <label for="rotate" class="block mb-0.5 text-sm font-medium">Rotate</label>
                        <Select
                            id="rotate"
                            v-model="selectedRotate"
                            :options="rotateOptions"
                            optionLabel="label"
                            optionValue="value"
                            :disabled="processing"
                            class="w-full"
                        />
                    </div>
                </div>

                <div class="settings-column">
                    <h2 class="text-base font-semibold mb-2 border-b border-gray-600 pb-1">Output/Encoding Settings</h2>

                    <div class="form-group">
                        <label for="video-output" class="block mb-0.5 text-sm font-medium">Output Path</label>
                        <small class="text-gray-500">Leave empty to play in terminal</small>
                        <div class="flex gap-2">
                            <InputText
                                id="video-output"
                                v-model="config.output_video_path"
                                placeholder="Save as... (optional)"
                                class="flex-1"
                                :disabled="processing"
                            />
                            <Button
                                type="button"
                                label="Browse"
                                :disabled="processing"
                                @click="browseOutputVideo"
                            />
                        </div>
                    </div>

                    <div class="form-group">
                        <label for="invert" class="block mb-0.5 text-sm font-medium">Background</label>
                        <small class="text-gray-500">Text color is inverted from background</small>
                        <ToggleButton
                            id="invert"
                            v-model="config.invert"
                            onLabel="Light Background"
                            offLabel="Dark Background"
                            :disabled="processing"
                            class="w-full"
                        />
                    </div>

<!--                    <div class="form-group">-->
<!--                        <label for="scale-down" class="block mb-0.5 text-sm font-medium">Num Threads</label>-->
<!--                        <small class="text-gray-500">Num Threads for parallel encoding</small>-->
<!--                        <InputNumber-->
<!--                            id="num-threads"-->
<!--                            v-model="config.num_threads"-->
<!--                            :min="1"-->
<!--                            :max="32"-->
<!--                            :step="1"-->
<!--                            :showButtons="true"-->
<!--                            :disabled="processing"-->
<!--                        />-->
<!--                    </div>-->

                    <div class="form-group">
                        <label for="max-fps" class="block mb-0.5 text-sm font-medium">Max FPS</label>
                        <small class="text-gray-500">Maximum frames per second for terminal playback</small>
                        <div class="flex items-center gap-2 my-2">
                            <Checkbox v-model="config.use_max_fps_for_output_video" :binary="true" :disabled="processing" />
                            <label for="use-max-fps" class="text-sm">Apply max FPS setting to video file output</label>
                        </div>
                        <InputNumber
                            id="max-fps"
                            v-model="config.max_fps"
                            :min="1"
                            :max="120"
                            :step="1"
                            :disabled="processing"
                        />
                    </div>
                </div>
            </div>

            <div class="mt-4">
                <Button
                    type="submit"
                    label="Asciify"
                    :loading="processing"
                    :disabled="!config.video_path || processing"
                    class="w-full"
                    size="large"
                />
            </div>

            <div v-if="processError" class="mt-4 p-3 bg-red-900/50 border border-red-700 rounded text-red-300">
                {{ processError }}
            </div>
        </form>
    </div>
</template>

<style scoped>
.video-settings {
    padding: 0.25rem;
    width: 100%;
    max-width: 900px;
    margin: 0 auto;
}

.settings-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.75rem;
}

@media (max-width: 700px) {
    .settings-grid {
        grid-template-columns: 1fr;
    }
}

.settings-column {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
}

.form-group {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
}

.form-group :deep(.p-inputnumber),
.form-group :deep(.p-select),
.form-group :deep(.p-inputtext) {
    width: 100%;
}

.form-group .flex.items-center.gap-2 {
    display: flex;
    align-items: center;
    gap: 0.5rem;
}
</style>
