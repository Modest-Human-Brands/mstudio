<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useRouter } from 'vue-router';
import ToolLayout from '../layouts/ToolLayout.vue';
import type { CertItem } from '../types';

const router = useRouter();
const certificates = ref<CertItem[]>([]);
const selectedCertIndex = ref<number | null>(null);
const statusMessage = ref('Scanning Windows Certificate Store...');
const isLoading = ref(false);

const selectedCert = computed(
  () => certificates.value.find((c) => c.index === selectedCertIndex.value) ?? null,
);

async function fetchCertificates() {
  isLoading.value = true;
  statusMessage.value = 'Accessing hardware tokens...';
  try {
    const res = await invoke<{ total_count: number; certificates: CertItem[] }>(
      'list_certificates',
    );
    certificates.value = res.certificates;
    if (res.total_count > 0 && selectedCertIndex.value === null) {
      selectedCertIndex.value = certificates.value[0]!.index;
    }
    statusMessage.value = `Found ${res.total_count} active certificate(s).`;
  } catch (err) {
    statusMessage.value = `Error: ${String(err)}`;
  } finally {
    isLoading.value = false;
  }
}

function copyToClipboard(text: string) {
  navigator.clipboard.writeText(text);
  statusMessage.value = 'Copied certificate data to clipboard!';
  setTimeout(() => {
    statusMessage.value = `Found ${certificates.value.length} active certificate(s).`;
  }, 2000);
}

onMounted(() => {
  fetchCertificates();
});
</script>

<template>
  <ToolLayout>
    <template #header>
      <div class="flex items-center gap-2 min-w-0">
        <span class="text-white/50 shrink-0 uppercase text-xs tracking-wider"
          >Hardware Security & DSC Manager</span
        >
      </div>
    </template>

    <div
      class="p-6 size-full mx-auto grid grid-cols-1 md:grid-cols-12 gap-6 items-start overflow-y-scroll"
    >
      <section
        class="md:col-span-6 bg-black/30 border border-white/10 rounded p-4 flex flex-col gap-3"
      >
        <div class="flex items-center justify-between border-b border-white/5 pb-2">
          <h3 class="text-xs font-semi-bold text-white/50 tracking-widest uppercase">
            Detected Tokens ({{ certificates.length }})
          </h3>
          <button
            class="px-2.5 py-1 bg-white/10 hover:bg-white/20 rounded text-xs transition-colors flex items-center gap-1.5"
            @click="fetchCertificates"
            :disabled="isLoading"
          >
            <span :class="isLoading ? 'animate-spin' : ''">↻</span>
            Refresh
          </button>
        </div>

        <div
          v-if="certificates.length === 0"
          class="text-sm text-white/40 py-8 text-center flex flex-col items-center gap-2"
        >
          <span class="text-2xl">🔌</span>
          <span>No hardware DSC tokens detected.</span>
          <span class="text-xs text-white/20"
            >Ensure your USB ePass2003 / mToken is plugged in.</span
          >
        </div>

        <div v-else class="flex flex-col gap-2 max-h-[500px] overflow-y-auto pr-1">
          <div
            v-for="cert in certificates"
            :key="cert.index"
            class="p-3 rounded border cursor-pointer transition-all flex items-start gap-3"
            :class="
              selectedCertIndex === cert.index
                ? 'border-primary-500 bg-primary-500/10'
                : 'border-white/10 bg-black/40 hover:border-white/20'
            "
            @click="selectedCertIndex = cert.index"
          >
            <div
              class="mt-1 w-2 h-2 rounded-full shrink-0"
              :class="
                selectedCertIndex === cert.index
                  ? 'bg-primary-500 shadow-sm shadow-primary-500'
                  : 'bg-white/20'
              "
            />
            <div class="min-w-0 grow">
              <div class="text-sm font-medium text-white truncate">{{ cert.subject }}</div>
              <div class="text-xs text-white/50 mt-0.5 truncate">Issuer: {{ cert.issuer }}</div>
              <div class="flex items-center gap-2 mt-2">
                <span
                  class="text-3xs font-mono uppercase px-1.5 py-0.5 rounded bg-white/5 border border-white/10 text-white/60"
                >
                  Index #{{ cert.index }}
                </span>
                <span
                  class="text-3xs font-mono uppercase px-1.5 py-0.5 rounded bg-white/5 border border-white/10 text-white/60"
                >
                  Chain: {{ cert.certificateChainDerHex.length }} CA(s)
                </span>
              </div>
            </div>
          </div>
        </div>
      </section>

      <section
        class="md:col-span-6 bg-black/30 border border-white/10 rounded p-4 flex flex-col gap-4"
      >
        <h3
          class="text-xs font-semi-bold text-white/50 tracking-widest uppercase border-b border-white/5 pb-2"
        >
          Certificate Inspector
        </h3>

        <div v-if="!selectedCert" class="text-sm text-white/30 py-12 text-center">
          Select a certificate from the list to view its cryptographic details.
        </div>

        <div v-else class="flex flex-col gap-4 text-xs">
          <div>
            <span class="text-2xs font-mono text-white/40 uppercase block mb-1">Subject Name</span>
            <div
              class="p-2.5 rounded bg-black/50 border border-white/5 font-mono text-white/90 break-words select-all"
            >
              {{ selectedCert.subject }}
            </div>
          </div>

          <div>
            <span class="text-2xs font-mono text-white/40 uppercase block mb-1">Issuer Name</span>
            <div
              class="p-2.5 rounded bg-black/50 border border-white/5 font-mono text-white/80 break-words select-all"
            >
              {{ selectedCert.issuer }}
            </div>
          </div>

          <div v-if="selectedCert.certificateChainDerHex.length > 0">
            <span class="text-2xs font-mono text-white/40 uppercase block mb-1">
              Trust Chain ({{ selectedCert.certificateChainDerHex.length }} Parent CAs)
            </span>
          </div>
        </div>
      </section>
    </div>

    <template #footer>
      <span class="text-xs text-white/40 font-mono">{{ statusMessage }}</span>
      <button
        class="px-3 py-1 rounded bg-white/10 hover:bg-white/20 text-white text-xs font-medium transition-colors"
        @click="router.back()"
      >
        BACK
      </button>
    </template>
  </ToolLayout>
</template>