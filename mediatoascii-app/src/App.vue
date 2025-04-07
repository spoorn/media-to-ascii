<script setup lang="ts">
import Video from "./video/Video.vue";
</script>

<template>
  <div class="settings mx-auto p-4">
    <Video/>
    <!--    <h1>Welcome to Tauri + Vue</h1>-->

    <!--    <div class="row">-->
    <!--      <a href="https://vitejs.dev" target="_blank">-->
    <!--        <img src="/vite.svg" class="logo vite" alt="Vite logo" />-->
    <!--      </a>-->
    <!--      <a href="https://tauri.app" target="_blank">-->
    <!--        <img src="/tauri.svg" class="logo tauri" alt="Tauri logo" />-->
    <!--      </a>-->
    <!--      <a href="https://vuejs.org/" target="_blank">-->
    <!--        <img src="./assets/vue.svg" class="logo vue" alt="Vue logo" />-->
    <!--      </a>-->
    <!--    </div>-->
    <!--    <p>Click on the Tauri, Vite, and Vue logos to learn more.</p>-->

    <!--    <form class="row" @submit.prevent="greet">-->
    <!--      <input id="greet-input" v-model="name" placeholder="Enter a name..." />-->
    <!--      <button type="submit">Greet</button>-->
    <!--    </form>-->
    <!--    <p>{{ greetMsg }}</p>-->
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

.settings {
  margin: 0;
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
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
  border-color: #396cd8;
}

button:active {
  border-color: #396cd8;
  background-color: #e8e8e8;
}

input,
button {
  outline: none;
}

#greet-input {
  margin-right: 5px;
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
}

</style>