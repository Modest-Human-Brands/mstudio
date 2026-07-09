import { createApp } from 'vue';
import { createRouter, createWebHashHistory } from 'vue-router';
import App from './App.vue';

import Home from './pages/index.vue';
import Stream from './pages/stream.vue';
// import Organize from './pages/organize.vue'
import Overlay from './pages/overlay.vue';
import Security from './pages/security.vue'; // <-- Import Security View
// import Sync from "./pages/sync.vue";

const routes = [
  { path: '/', name: 'home', component: Home },
  { path: '/stream', name: 'stream', component: Stream },
  // { path: '/organize', name: 'organize', component: Organize },
  { path: '/overlay', name: 'overlay', component: Overlay },
  { path: '/security', name: 'security', component: Security },
  // { path: "/sync", name: "sync", component: Sync },
];

export const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

createApp(App).use(router).mount('#app');