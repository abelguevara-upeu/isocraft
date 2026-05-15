import { createApp } from "vue";
import App from "./App.vue";

import "./index.css";

const app = createApp(App);

// Future global configurations (pinia, router, etc) can go here

app.mount("#app");
