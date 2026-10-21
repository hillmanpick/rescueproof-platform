<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { Leaf, ArrowUpRight, Menu, X, LogOut } from "lucide-vue-next";
import { session, refreshSession, post, errorMessage } from "./api";
import { locale, languageOptions, isLocale, setLocale, t } from "./i18n";
const menu = ref(false),
  connectionError = ref("");
const actionError = ref<unknown>(null);
const router = useRouter();
onMounted(() =>
  refreshSession().catch(() => {
    connectionError.value = "common.serviceUnavailable";
  }),
);
async function logout() {
  actionError.value = null;
  try {
    await post("/auth/logout");
    session.user = null;
    session.organizations = [];
    menu.value = false;
    router.push("/");
  } catch (error) {
    actionError.value = error;
  }
}
function changeLanguage(event: Event) {
  const value = (event.target as HTMLSelectElement).value;
  if (isLocale(value)) setLocale(value);
}
</script>
<template>
  <div class="notice">
    {{ t("notice") }}<span>{{ t("common.motto") }}</span>
  </div>
  <header class="site-header">
    <div class="container header-inner">
      <RouterLink to="/" class="brand" :aria-label="t('common.home')"
        ><span class="brand-icon"><Leaf :size="23" /></span>Rescue<span
          class="brand-light"
          >Proof</span
        ><span class="brand-dot">®</span></RouterLink
      >
      <button
        class="mobile-menu icon-button"
        @click="menu = !menu"
        :aria-label="t(menu ? 'common.closeMenu' : 'common.openMenu')"
        :aria-expanded="menu"
        aria-controls="main-navigation"
      >
        <X v-if="menu" /><Menu v-else />
      </button>
      <nav
        id="main-navigation"
        :class="{ expanded: menu }"
        @click="menu = false"
      >
        <RouterLink to="/">{{ t("discover") }}</RouterLink
        ><RouterLink to="/#how">{{ t("how") }}</RouterLink
        ><RouterLink to="/workspace">{{ t("workspace") }}</RouterLink
        ><RouterLink v-if="session.user?.is_admin" to="/admin">{{
          t("admin")
        }}</RouterLink>
      </nav>
      <div class="header-actions">
        <label class="language-picker"
          ><span class="sr-only">{{ t("common.language") }}</span
          ><select :value="locale.value" @change="changeLanguage">
            <option
              v-for="option in languageOptions"
              :key="option.value"
              :value="option.value"
            >
              {{ option.label }}
            </option>
          </select></label
        ><template v-if="session.user"
          ><RouterLink to="/workspace" class="account-name">{{
            session.user.name
          }}</RouterLink
          ><button
            class="icon-button"
            :aria-label="t('logout')"
            @click="logout"
          >
            <LogOut :size="18" /></button></template
        ><RouterLink v-else to="/login" class="login-link">{{
          t("login")
        }}</RouterLink
        ><RouterLink to="/share" class="button small"
          >{{ t("share") }} <ArrowUpRight :size="16"
        /></RouterLink>
      </div>
    </div>
  </header>
  <div v-if="connectionError" class="connection-banner" role="status">
    {{ t(connectionError) }}
    <button
      @click="
        connectionError = '';
        refreshSession().catch(() => (connectionError = 'common.retryService'));
      "
    >
      {{ t("common.retryConnection") }}
    </button>
  </div>
  <div v-if="actionError" class="connection-banner" role="alert">
    {{ errorMessage(actionError) }}
  </div>
  <main><RouterView :key="$route.path" /></main>
  <footer>
    <div class="container footer-inner">
      <div>
        <RouterLink class="brand" to="/"
          ><Leaf :size="23" /> RescueProof</RouterLink
        >
        <p>{{ t("footer") }}</p>
      </div>
      <div class="footer-meta">
        <span>{{ t("footerMeta") }}</span
        ><small>RescueProof · {{ t("common.pilot") }}</small>
      </div>
    </div>
    <div class="container footer-bottom">
      © {{ new Date().getFullYear() }} RescueProof
      <span>{{ t("footerSafety") }}</span>
    </div>
  </footer>
</template>
