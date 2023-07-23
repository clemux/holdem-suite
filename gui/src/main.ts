import { createApp } from "vue";
import "./styles.css";
import App from "./App.vue";
import { Quasar } from 'quasar'

// Import icon libraries
import '@quasar/extras/material-icons/material-icons.css'

// Import Quasar css
import 'quasar/src/css/index.sass'

createApp(App).use(Quasar, {
    plugins: {},
}).mount("#app");
