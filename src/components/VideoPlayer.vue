<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from 'vue';
import videojs from 'video.js';
import 'video.js/dist/video-js.css';

const props = defineProps<{
  src: string;
}>();

const videoElement = ref<HTMLVideoElement | null>(null);
let player: ReturnType<typeof videojs> | null = null;

onMounted(() => {
  if (videoElement.value) {
    player = videojs(
      videoElement.value,
      {
        autoplay: true,
        muted: true,
        controls: false,
        preload: 'auto',
      },
      () => {
        player?.src({ src: props.src, type: 'application/x-mpegURL' });
      },
    );
  }
});

onUnmounted(() => {
  if (player) {
    player.dispose();
  }
});

watch(
  () => props.src,
  (newSrc) => {
    if (player && newSrc) {
      player.src({ src: newSrc, type: 'application/x-mpegURL' });
      player.play();
    }
  },
);
</script>

<template>
  <video ref="videoElement" class="video-js vjs-default-skin vjs-fill object-cover"></video>
</template>