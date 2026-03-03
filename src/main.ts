import { createApp } from "vue";
import App from "./App.vue";
import DobruniaUI from "dobruniaui-vue";
import "dobruniaui-vue/styles.css";

const app = createApp(App);
app.use(DobruniaUI);
app.mount("#app");
