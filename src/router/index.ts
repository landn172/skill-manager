import { createRouter, createWebHistory } from "vue-router";
import Marketplace from "@/pages/Marketplace.vue";
import Installed from "@/pages/Installed.vue";
import Settings from "@/pages/Settings.vue";
import CreateSkill from "@/pages/CreateSkill.vue";

const routes = [
  {
    path: "/",
    redirect: "/marketplace",
  },
  {
    path: "/marketplace",
    name: "Marketplace",
    component: Marketplace,
  },
  {
    path: "/installed",
    name: "Installed",
    component: Installed,
  },
  {
    path: "/settings",
    name: "Settings",
    component: Settings,
  },
  {
    path: "/create",
    name: "CreateSkill",
    component: CreateSkill,
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;
