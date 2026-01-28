import { path } from "@tauri-apps/api";
import { createWebHashHistory, createRouter } from "vue-router";

const routes = [
    { path: "/", component: () => import("../views/ListView.vue") },
    { path: "/device/:id/:name", component: () => import("../views/DeviceView.vue") },
    { path: "/device/:id/:name/widget", component: () => import("../views/FloatingView.vue") },
    { path: "/settings", component: () => import("../views/Settings.vue") },
    { path: "/about", component: () => import("../views/About.vue") },
    {path: "/splash", component: () => import("../views/SplashView.vue")},
];

const router = createRouter({
    history: createWebHashHistory(),
    routes,
});

export default router;