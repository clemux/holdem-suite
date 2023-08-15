import {createApp} from "vue";
import "./styles.css";
import PlayerHud from "./components/PlayerHud.vue";
import {Quasar} from 'quasar'

// Import icon libraries
import '@quasar/extras/material-icons/material-icons.css'

// Import Quasar css
import 'quasar/src/css/index.sass'

createApp(PlayerHud).use(Quasar, {
    plugins: {},
}).mount("#hud");
