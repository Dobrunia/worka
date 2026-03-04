import { createApp } from "vue";
import App from "./App.vue";
import "dobruniaui-vue/styles.css";
import { getCurrentWindow } from "@tauri-apps/api/window";

const app = createApp(App);
app.mount("#app");

// Show window only after Vue has mounted and CSS is applied.
// The __TAURI_INTERNALS__ guard prevents this from running in browser-only dev mode.
if ("__TAURI_INTERNALS__" in window) {
  setTimeout(() => getCurrentWindow().show(), 50);
}
