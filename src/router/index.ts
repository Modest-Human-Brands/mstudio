import { createRouter, createWebHistory } from 'vue-router'

import Home from '@/pages/index.vue'
import Stream from '@/pages/stream.vue'
// import Organize from '@/pages/organize.vue'
import Overlay from '@/pages/overlay.vue'
// import Sync from "@/pages/sync.vue";
import CryptoToken from '@/pages/crypto-token.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    { path: '/', name: 'home', component: Home },
    { path: '/stream', name: 'stream', component: Stream },
    // { path: '/organize', name: 'organize', component: Organize },
    { path: '/overlay', name: 'overlay', component: Overlay },
    // { path: "/sync", name: "sync", component: Sync },
    { path: '/crypto-token', name: 'crypto-token', component: CryptoToken },
  ],
})

export default router