<script setup lang="ts">
import { ref } from "vue";
import Video from "./video/Video.vue";
import ImagePanel from "./image/Image.vue";

const activeTab = ref<"video" | "image">("video");

const tabs = [
  { id: "video", label: "VIDEO", icon: "🎬" },
  { id: "image", label: "IMAGE", icon: "🖼️" },
];
</script>

<template>
  <div
    class="relative min-h-screen w-full overflow-hidden bg-[var(--color-bg-dark)] text-slate-100 font-sans selection:bg-cyan-500/30"
  >
    <!-- Animated Background Elements -->
    <div class="pointer-events-none absolute inset-0 overflow-hidden">
      <div
        class="absolute -top-[20%] -left-[10%] h-[70vh] w-[70vh] rounded-full bg-purple-900/20 blur-[120px]"
      ></div>
      <div
        class="absolute top-[40%] -right-[10%] h-[60vh] w-[60vh] rounded-full bg-cyan-900/20 blur-[100px]"
      ></div>
      <div
        class="absolute bottom-[-10%] left-[20%] h-[50vh] w-[50vh] rounded-full bg-pink-900/10 blur-[100px]"
      ></div>
    </div>

    <div class="relative z-10 flex h-screen flex-col md:flex-row">
      <!-- Sidebar Navigation -->
      <aside
        class="glass-panel z-20 flex w-full flex-col justify-between border-r border-white/5 p-6 md:w-64 md:h-full"
      >
        <div>
          <div class="mb-10 flex items-center gap-3">
            <div
              class="flex h-10 w-10 items-center justify-center rounded-lg bg-gradient-to-br from-cyan-500 to-blue-600 shadow-lg shadow-cyan-500/20"
            >
              <span class="text-xl font-bold text-white">M</span>
            </div>
            <div>
              <h1 class="font-mono text-lg font-bold tracking-wider text-white">
                M2A<span class="text-cyan-400">.STUDIO</span>
              </h1>
              <p class="text-[10px] uppercase tracking-widest text-slate-500">
                v0.1.0
              </p>
            </div>
          </div>

          <nav class="space-y-2">
            <button
              v-for="tab in tabs"
              :key="tab.id"
              @click="activeTab = tab.id as 'video' | 'image'"
              class="group flex w-full items-center gap-3 rounded-xl px-4 py-3 text-sm font-medium transition-all duration-300"
              :class="
                activeTab === tab.id
                  ? 'bg-white/10 text-cyan-400 shadow-inner'
                  : 'text-slate-400 hover:bg-white/5 hover:text-slate-200'
              "
            >
              <span
                class="text-lg opacity-80 group-hover:scale-110 transition-transform"
                >{{ tab.icon }}</span
              >
              {{ tab.label }}
            </button>
          </nav>
        </div>

        <div class="mt-auto pt-6 border-t border-white/5">
          <a
            href="https://github.com/spoorn/media-to-ascii"
            target="_blank"
            class="flex items-center gap-2 rounded-lg px-4 py-2 text-xs text-slate-500 transition hover:text-cyan-400"
          >
            <svg
              class="h-4 w-4"
              fill="currentColor"
              viewBox="0 0 24 24"
              aria-hidden="true"
            >
              <path
                fill-rule="evenodd"
                d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"
                clip-rule="evenodd"
              />
            </svg>
            <span>Documentation</span>
          </a>
        </div>
      </aside>

      <!-- Main Content Area -->
      <main class="flex-1 overflow-y-auto p-4 md:p-8">
        <div class="mx-auto max-w-5xl">
          <transition name="fade" mode="out-in">
            <div v-if="activeTab === 'video'" key="video">
              <Video />
            </div>
            <div v-else key="image">
              <ImagePanel />
            </div>
          </transition>
        </div>
      </main>
    </div>
  </div>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease, transform 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
  transform: translateY(10px);
}
</style>
