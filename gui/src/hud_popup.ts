import { createApp } from "vue";
import "./styles.css";
import PlayerHudPopup from "./components/PlayerHudPopup.vue";
import { Quasar } from 'quasar'
import { invoke } from '@tauri-apps/api/tauri'

// Import icon libraries
import '@quasar/extras/material-icons/material-icons.css'

// Import Quasar css
import 'quasar/src/css/index.sass'

createApp(PlayerHudPopup).use(Quasar, {
    plugins: {},
}).mount("#app");
