import { createApp } from "vue";
import "./styles.css";
import App from "./App.vue";
import { Quasar } from 'quasar'
import { invoke } from '@tauri-apps/api/tauri'

// Import icon libraries
import '@quasar/extras/material-icons/material-icons.css'

// Import Quasar css
import 'quasar/src/css/index.sass'

createApp(App).use(Quasar, {
    plugins: {},
}).mount("#app");

document.addEventListener('DOMContentLoaded', () => {
  // This will wait for the window to load, but you could
  // run this function on whatever trigger you want
  invoke('close_splashscreen')
})