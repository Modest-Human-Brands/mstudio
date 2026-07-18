<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import VideoPlayer from '../components/VideoPlayer.vue';
import { useRouter } from 'vue-router';
import ToolLayout from '../layouts/ToolLayout.vue';

const router = useRouter();

const streamKey = ref('');
const statusMessage = ref('Fetching native hardware states...');
const isStreaming = ref(false);
const isPreviewReady = ref(false);
const isLoadingDevices = ref(true);

interface MediaDevices {
  video: string[];
  audio: string[];
}
const cameras = ref<string[]>([]);
const mics = ref<string[]>([]);
const selectedCamera = ref<string>('');
const selectedMic = ref<string>('');
const previewTimestamp = ref(Date.now());

const previewUrl = computed(() => {
  if (!selectedCamera.value) return '';
  return `http://127.0.0.1:8720/api/preview/index.m3u8?t=${previewTimestamp.value}`;
});

watch(selectedCamera, () => {
  previewTimestamp.value = Date.now();
});

async function fetchNativeDevices() {
  isLoadingDevices.value = true;
  try {
    const devices = await invoke<MediaDevices>('get_devices');
    cameras.value = devices.video;
    mics.value = devices.audio;

    if (cameras.value.length > 0) selectedCamera.value = cameras.value[0];
    if (mics.value.length > 0) selectedMic.value = mics.value[0];

    statusMessage.value = 'Hardware synchronization complete.';
  } catch (err) {
    statusMessage.value = `Hardware scan failed: ${String(err)}`;
  } finally {
    isLoadingDevices.value = false;
  }
}

onMounted(async () => {
  await fetchNativeDevices();
});

onUnmounted(() => {
  invoke('stop_preview');
});

async function handleStartStream() {
  if (!selectedCamera.value || !selectedMic.value) return;

  try {
    isStreaming.value = true;
    statusMessage.value = streamKey.value
      ? 'Reconfiguring hardware for live broadcast...'
      : 'Generating Local Stream...';

    const response = await invoke<string>('start_stream', {
      streamKey: streamKey.value,
      videoDevice: selectedCamera.value,
      audioDevice: selectedMic.value,
    });

    statusMessage.value = response;

    let ready = false;
    let attempts = 0;
    while (!ready && attempts < 30 && isStreaming.value) {
      try {
        const res = await fetch('http://127.0.0.1:8720/api/preview/index.m3u8', { method: 'HEAD' });
        if (res.ok) {
          ready = true;
          break;
        }
      } catch (err) {
        /* Ignore connection refused while FFmpeg boots */
      }
      await new Promise((resolve) => setTimeout(resolve, 500));
      attempts++;
    }

    if (ready && isStreaming.value) {
      isPreviewReady.value = true;
    } else if (isStreaming.value) {
      isStreaming.value = false;
      statusMessage.value = 'Failed to generate local stream (Timeout).';
    }
  } catch (error) {
    isStreaming.value = false;
    statusMessage.value = `Process Failed: ${error}`;
  }
}
</script>

<template>
  <ToolLayout>
    <template #header>
      <div class="flex items-center justify-between w-full relative z-20">
        <h1 class="text-base font-bold text-white tracking-wide drop-shadow-md">
          Live Broadcast Controller
        </h1>
        <button
          @click="router.back()"
          class="text-white/80 hover:text-white transition-colors cursor-pointer p-1 bg-black/20 rounded-full backdrop-blur-sm"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            class="size-5"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            stroke-width="2"
          >
            <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>
    </template>

    <div
      class="relative w-full h-full overflow-hidden bg-black flex flex-col items-center justify-end"
    >
      <VideoPlayer
        v-if="isStreaming && isPreviewReady"
        :src="previewUrl"
        class="absolute inset-0 w-full h-full z-0 transform scale-x-[-1]"
      />

      <div
        v-if="isStreaming && isPreviewReady"
        class="absolute top-6 left-6 flex items-center bg-black/60 backdrop-blur-md px-4 py-2 rounded-full z-10 gap-3 shadow-lg border border-white/10"
      >
        <div
          class="size-3 rounded-full animate-pulse"
          :class="
            streamKey
              ? 'bg-red-500 shadow-[0_0_15px_rgba(239,68,68,0.8)]'
              : 'bg-green-500 shadow-[0_0_15px_rgba(34,197,94,0.8)]'
          "
        ></div>
        <span class="text-white/90 font-bold text-xs tracking-widest uppercase">{{
          streamKey ? 'Live' : 'Preview'
        }}</span>
      </div>

      <div
        v-if="isStreaming && !isPreviewReady"
        class="absolute inset-0 flex flex-col items-center justify-center bg-black/80 backdrop-blur-md z-0 gap-4"
      >
        <div
          class="size-8 border-4 border-primary-500 border-t-transparent rounded-full animate-spin"
        ></div>
        <span class="text-white/90 font-semibold text-lg tracking-widest uppercase"
          >Buffering Stream...</span
        >
      </div>

      <div
        v-else-if="!selectedCamera"
        class="absolute inset-0 flex flex-col items-center justify-center z-0 gap-3"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          class="size-10 text-white/20"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z"
          />
        </svg>
        <span class="text-white/40 font-medium text-sm uppercase tracking-widest"
          >Select Camera to Start</span
        >
      </div>

      <div
        v-else-if="!isStreaming && selectedCamera"
        class="absolute inset-0 flex flex-col items-center justify-center z-0 gap-3"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          class="size-10 text-white/40"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="1.5"
            d="M15.75 10.5l4.72-4.72a.75.75 0 011.28.53v11.38a.75.75 0 01-1.28.53l-4.72-4.72M4.5 18.75h9a2.25 2.25 0 002.25-2.25v-9a2.25 2.25 0 00-2.25-2.25h-9A2.25 2.25 0 002.25 7.5v9a2.25 2.25 0 002.25 2.25z"
          />
        </svg>
        <span class="text-white/60 font-medium text-sm uppercase tracking-widest"
          >Camera Ready - Start Stream for Preview</span
        >
      </div>

      <div
        class="relative z-10 w-full max-w-4xl bg-gray-900/80 backdrop-blur-xl border border-white/10 rounded-2xl p-2 mb-6 grid grid-cols-1 sm:grid-cols-3 gap-2 shadow-2xl mx-4"
      >
        <div
          class="relative flex items-center bg-white/10 hover:bg-white/20 transition-colors rounded-xl overflow-hidden"
        >
          <div class="pl-3 py-2 pr-1 flex items-center justify-center pointer-events-none">
            <svg class="size-4 text-white/80" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="1.5"
                d="M12 18.75a6 6 0 006-6v-1.5m-6 7.5a6 6 0 01-6-6v-1.5m6 7.5v3.75m-3.75 0h7.5M12 15.75a3 3 0 01-3-3V4.5a3 3 0 116 0v8.25a3 3 0 01-3 3z"
              />
            </svg>
          </div>
          <select
            v-model="selectedMic"
            :disabled="isStreaming || isLoadingDevices"
            class="bg-transparent py-2.5 pl-2 pr-8 text-white text-xs font-medium outline-none disabled:opacity-50 appearance-none w-full cursor-pointer truncate"
          >
            <option v-if="mics.length === 0" value="">No microphone</option>
            <option v-for="mic in mics" :key="mic" :value="mic" class="bg-dark-700 text-white">
              {{ mic }}
            </option>
          </select>
        </div>

        <div
          class="relative flex items-center bg-white/10 hover:bg-white/20 transition-colors rounded-xl overflow-hidden"
        >
          <div class="pl-3 py-2 pr-1 flex items-center justify-center pointer-events-none">
            <svg class="size-4 text-white/80" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="1.5"
                d="M15.75 10.5l4.72-4.72a.75.75 0 011.28.53v11.38a.75.75 0 01-1.28.53l-4.72-4.72M4.5 18.75h9a2.25 2.25 0 002.25-2.25v-9a2.25 2.25 0 00-2.25-2.25h-9A2.25 2.25 0 002.25 7.5v9a2.25 2.25 0 002.25 2.25z"
              />
            </svg>
          </div>
          <select
            v-model="selectedCamera"
            :disabled="isStreaming || isLoadingDevices"
            class="bg-transparent py-2.5 pl-2 pr-8 text-white text-xs font-medium outline-none disabled:opacity-50 appearance-none w-full cursor-pointer truncate"
          >
            <option v-if="cameras.length === 0" value="">No camera</option>
            <option v-for="cam in cameras" :key="cam" :value="cam" class="bg-dark-700 text-white">
              {{ cam }}
            </option>
          </select>
        </div>

        <div
          class="relative flex items-center bg-white/10 focus-within:bg-white/20 transition-colors rounded-xl w-full overflow-hidden border border-transparent focus-within:border-primary-500/50"
        >
          <div class="pl-3 py-2 pr-1 flex items-center justify-center pointer-events-none">
            <svg class="size-4 text-white/80" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="1.5"
                d="M15.75 5.25a3 3 0 013 3m3 0a6 6 0 01-7.029 5.912c-.563-.097-1.159.026-1.563.43L10.5 17.25H8.25v2.25H6v2.25H2.25v-2.818c0-.597.237-1.17.659-1.591l6.499-6.499c.404-.404.527-1 .43-1.563A6 6 0 1121.75 8.25z"
              />
            </svg>
          </div>
          <input
            v-model="streamKey"
            type="password"
            placeholder="Stream Key..."
            :disabled="isStreaming"
            class="bg-transparent py-2.5 pl-2 pr-4 text-white text-xs font-mono outline-none disabled:opacity-50 w-full placeholder-white/40 truncate"
          />
        </div>
      </div>
    </div>

    <template #footer>
      <div class="relative flex items-center justify-between w-full">
        <div class="flex items-center gap-2 text-white/70 font-medium w-full truncate mr-4">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            class="size-5 shrink-0"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            stroke-width="2"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
          <span
            class="text-xs font-mono truncate max-w-full"
            :class="isStreaming ? 'text-primary-400 font-bold' : ''"
          >
            {{ statusMessage }}
          </span>
        </div>

        <div class="flex items-center gap-3 shrink-0">
          <button
            class="px-5 py-2 rounded-full border border-white/20 hover:border-white/50 hover:bg-white/10 text-white text-xs font-semibold transition-all bg-transparent cursor-pointer tracking-wide"
            @click="router.back()"
            :disabled="isStreaming"
          >
            Cancel
          </button>
          <button
            class="px-6 py-2 rounded-full text-white text-xs font-bold transition-all tracking-wider shadow-lg flex items-center gap-2 uppercase"
            :class="
              isStreaming
                ? streamKey
                  ? 'bg-red-600 shadow-red-500/20 cursor-default'
                  : 'bg-green-600 shadow-green-500/20 cursor-default'
                : streamKey
                  ? 'bg-primary-600 hover:bg-primary-500 cursor-pointer shadow-primary-500/20'
                  : 'bg-white/20 hover:bg-white/30 cursor-pointer'
            "
            :disabled="isStreaming"
            @click="isStreaming ? () => {} : handleStartStream()"
          >
            <div v-if="isStreaming" class="size-2 rounded-full bg-white animate-pulse"></div>
            <svg
              v-else
              xmlns="http://www.w3.org/2000/svg"
              class="size-4"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
              stroke-width="2.5"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                d="M5.25 5.653c0-.856.917-1.398 1.667-.986l11.54 6.347a1.125 1.125 0 010 1.972l-11.54 6.347a1.125 1.125 0 01-1.667-.986V5.653z"
              />
            </svg>
            {{
              isStreaming ? (streamKey ? 'Live' : 'Previewing') : streamKey ? 'Go Live' : 'Preview'
            }}
          </button>
        </div>
      </div>
    </template>
  </ToolLayout>
</template>

<style scoped>
select {
  background-image: url("data:image/svg+xml;charset=utf-8,%3Csvg xmlns='http://www.w3.org/2000/svg' fill='none' viewBox='0 0 20 20'%3E%3Cpath stroke='%23ffffff' stroke-linecap='round' stroke-linejoin='round' stroke-width='1.5' d='m6 8 4 4 4-4'/%3E%3C/svg%3E");
  background-position: right 0.75rem center;
  background-repeat: no-repeat;
  background-size: 1.2em 1.2em;
}

select option {
  background-color: #1a1a1a;
  color: white;
}

:deep([data-vjs-player]) {
  width: 100%;
  height: 100%;
}

:deep(.video-js) {
  width: 100% !important;
  height: 100% !important;
  background-color: transparent !important;
}

:deep(.vjs-tech) {
  object-fit: cover !important;
}
</style>