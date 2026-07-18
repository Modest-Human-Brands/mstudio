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
const showInspector = ref(false);

const selectedCert = computed(
  () => certificates.value.find((c) => c.index === selectedCertIndex.value) ?? null,
);

function extractCommonName(dn: string): string {
  if (!dn) return '';

  const cnMatch = dn.match(/(?:^|,\s*)CN\s*=\s*([^,]+)/i);
  if (cnMatch && cnMatch[1]) {
    return cnMatch[1].replace(/^["']|["']$/g, '').trim();
  }

  const oMatch = dn.match(/(?:^|,\s*)O\s*=\s*([^,]+)/i);
  if (oMatch && oMatch[1]) {
    return oMatch[1].replace(/^["']|["']$/g, '').trim();
  }

  const parts = dn.split(',');
  if (parts[0] && parts[0].includes('=')) {
    return parts[0].split('=')[1]!.trim();
  }

  return dn;
}

function formatSubjectTitle(dn: string): string {
  const cleanName = extractCommonName(dn) || 'Unknown Identity';
  return `${cleanName} (Windows Digital ID)`;
}

function formatIssuerSub(dn: string): string {
  const cleanIssuer = extractCommonName(dn) || 'Unknown Certificate Authority';
  return `Issued by: ${cleanIssuer}`;
}

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

// function handleContinue() {
//   if (selectedCert.value) {
//     statusMessage.value = `Proceeding with: ${extractCommonName(selectedCert.value.subject)}`;
//   }
// }

onMounted(() => {
  fetchCertificates();
});
</script>

<template>
  <ToolLayout>
    <template #header>
      <div class="flex items-center justify-between w-full">
        <h1 class="text-base font-bold text-white tracking-wide">Sign with a Digital ID</h1>
        <button
          @click="router.back()"
          class="text-white/60 hover:text-white transition-colors cursor-pointer p-1"
          aria-label="Close"
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

    <div class="p-5 w-full mx-auto flex flex-col gap-6 overflow-y-auto h-full text-sm text-white">
      <div class="flex flex-col gap-3">
        <div
          v-if="certificates.length === 0"
          class="text-sm text-white/40 py-12 border border-dashed border-white/10 rounded-xl text-center flex flex-col items-center gap-2 bg-black/10"
        >
          <span class="text-3xl">🔌</span>
          <span class="font-medium text-white/70">No hardware DSC tokens detected.</span>
          <span class="text-xs text-white/40">
            Ensure your USB ePass2003 / mToken is securely plugged in.
          </span>
        </div>
        <div v-else class="flex flex-col gap-2">
          <div
            v-for="cert in certificates"
            :key="cert.index"
            class="p-4 rounded-lg transition-all flex items-start justify-between gap-4 border"
            :class="
              selectedCertIndex === cert.index
                ? 'border-white/10 bg-white/[0.02]'
                : 'border-transparent bg-transparent hover:bg-white/[0.01]'
            "
          >
            <div
              class="flex items-start gap-4 min-w-0 grow cursor-pointer"
              @click="selectedCertIndex = cert.index"
            >
              <div class="mt-1.5 shrink-0 flex items-center justify-center">
                <div
                  class="size-[18px] rounded-full border-2 flex items-center justify-center transition-all"
                  :class="
                    selectedCertIndex === cert.index
                      ? 'border-primary-500 bg-transparent'
                      : 'border-white/40'
                  "
                >
                  <div
                    v-if="selectedCertIndex === cert.index"
                    class="size-[8px] rounded-full bg-primary-500"
                  />
                </div>
              </div>

              <div class="shrink-0 text-white/70 bg-white/5 p-2 rounded border border-white/10">
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  class="size-6"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                  stroke-width="1.5"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
                  />
                  <circle cx="12" cy="9" r="2" class="stroke-primary-400" stroke-width="2" />
                </svg>
              </div>

              <div class="min-w-0 grow">
                <div class="text-sm font-semibold text-white truncate">
                  {{ formatSubjectTitle(cert.subject) }}
                </div>
                <div class="text-xs text-white/50 mt-0.5 truncate leading-relaxed">
                  {{ formatIssuerSub(cert.issuer) }}
                </div>
                <div class="flex items-center gap-3 mt-2">
                  <span
                    class="text-[10px] font-mono tracking-wider uppercase px-2 py-0.5 rounded bg-white/5 text-white/40 border border-white/5"
                  >
                    Slot Key #{{ cert.index }}
                  </span>
                </div>
              </div>
            </div>

            <div class="shrink-0 self-start pt-1">
              <button
                @click="showInspector = !showInspector"
                class="text-primary-400 hover:text-primary-300 transition-colors text-xs font-semibold cursor-pointer underline underline-offset-4"
              >
                {{
                  showInspector && selectedCertIndex === cert.index
                    ? 'Hide Details'
                    : 'View Details'
                }}
              </button>
            </div>
          </div>
        </div>
      </div>

      <transition name="fade">
        <div
          v-if="showInspector && selectedCert"
          class="bg-black/20 border border-white/10 rounded-xl p-4 flex flex-col gap-4 text-xs animate-fadeIn"
        >
          <div class="flex items-center justify-between border-b border-white/5 pb-2">
            <h3 class="font-bold text-white/60 uppercase tracking-wider text-[11px]">
              Certificate Signature & Trust Chain Details
            </h3>
          </div>
          <div>
            <span class="text-[10px] font-mono text-white/40 uppercase block mb-1">
              Full Subject Distinguished Name
            </span>
            <div
              class="p-2.5 rounded bg-black/40 border border-white/5 font-mono text-white/80 break-all select-all"
            >
              {{ selectedCert.subject }}
            </div>
          </div>
          <div>
            <span class="text-[10px] font-mono text-white/40 uppercase block mb-1"
              >Authority Certificate Issuer</span
            >
            <div
              class="p-2.5 rounded bg-black/40 border border-white/5 font-mono text-white/80 break-all select-all"
            >
              {{ selectedCert.issuer }}
            </div>
          </div>
          <div v-if="selectedCert.certificateChainDerHex.length > 0">
            <span class="text-[10px] font-mono text-white/40 uppercase block mb-1"
              >Cryptographic Trust Path Verification</span
            >
            <div class="p-2.5 rounded bg-black/40 border border-white/5 font-mono text-white/60">
              Verified secure via {{ selectedCert.certificateChainDerHex.length }} intermediate
              native Certificate Authorities.
            </div>
          </div>
        </div>
      </transition>
    </div>

    <template #footer>
      <div class="flex items-center gap-2 text-white/50 font-medium">
        <button
          class="hover:text-white transition-colors cursor-pointer"
          aria-label="Help Helpdesk"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            class="size-5"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            stroke-width="2"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="M8.228 9c.549-1.165 2.03-2 3.772-2 2.21 0 4 1.343 4 3 0 1.4-1.278 2.575-3.006 2.907-.542.104-.994.54-.994 1.093m0 3h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
        </button>
        <span class="text-xs font-mono truncate max-w-[240px] md:max-w-xs">{{
          statusMessage
        }}</span>
      </div>

      <div class="flex items-center gap-2.5">
        <button
          class="px-6 py-1.5 rounded-full border border-white/40 hover:border-white text-white text-xs font-semibold transition-all bg-transparent cursor-pointer tracking-wide"
          @click="router.back()"
        >
          Cancel
        </button>

        <button
          class="px-6 py-1.5 border border-white/40 hover:border-white rounded-full text-xs font-medium transition-colors bg-transparent cursor-pointer flex items-center gap-1.5 shrink-0"
          @click="fetchCertificates"
          :disabled="isLoading"
        >
          <span :class="isLoading ? 'animate-spin' : ''" class="text-xs">↻</span>
          Refresh
        </button>

        <!-- <button class="px-6 py-1.5 rounded-full text-white text-xs font-semibold transition-all tracking-wide shadow-sm"
          :class="selectedCertIndex !== null ? 'bg-primary-500 hover:bg-primary-600 cursor-pointer' : 'bg-white/10 text-white/30 cursor-not-allowed'"
          :disabled="selectedCertIndex === null" @click="handleContinue">
          Continue
        </button> -->
      </div>
    </template>
  </ToolLayout>
</template>

<style scoped>
.animate-fadeIn {
  animation: fadeIn 0.2s ease-out forwards;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(4px);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>