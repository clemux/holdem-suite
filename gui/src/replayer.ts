import {createApp} from "vue";
import "./styles.css";
import {Quasar} from 'quasar'

// Import icon libraries
import '@quasar/extras/material-icons/material-icons.css'

// Import Quasar css
import 'quasar/src/css/index.sass'
import ReplayerApp from "./components/ReplayerApp.vue";

createApp(ReplayerApp).use(Quasar, {
    plugins: {},
    config: {
        dark: true,
    }
}).mount("#app");
