import { createApp } from "vue";
import * as Vue from "vue"; // Import Vue to expose it
import App from "./App.vue";
import Antd from 'ant-design-vue';
import 'ant-design-vue/dist/reset.css';
import { createPinia } from 'pinia';
import { router } from './router';
import { i18n } from './i18n';

// Expose Vue globally for plugins
(window as any).Vue = Vue;

const app = createApp(App);

app.use(Antd);
app.use(createPinia());
app.use(router);
app.use(i18n);

app.mount("#app");
