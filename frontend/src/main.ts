import { createApp } from "vue";
import { createRouter, createWebHistory } from "vue-router";
import App from "./App.vue";
import Home from "./views/Home.vue";
import Auth from "./views/Auth.vue";
import Workspace from "./views/Workspace.vue";
import CreateBatch from "./views/CreateBatch.vue";
import Detail from "./views/Detail.vue";
import Admin from "./views/Admin.vue";
import "./style.css";
const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: "/", component: Home },
    { path: "/login", component: Auth },
    { path: "/register", component: Auth },
    { path: "/workspace", component: Workspace },
    { path: "/share", component: CreateBatch },
    { path: "/batches/:id", component: Detail },
    { path: "/admin", component: Admin },
    { path: "/:pathMatch(.*)*", component: Home },
  ],
  scrollBehavior: (to) =>
    to.hash ? { el: to.hash, top: 16, behavior: "smooth" } : { top: 0 },
});
createApp(App).use(router).mount("#app");
