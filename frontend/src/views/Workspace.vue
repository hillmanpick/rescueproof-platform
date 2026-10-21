<script setup lang="ts">
import { onMounted, ref } from "vue";
import { ArrowUpRight, Leaf, Bell, ShieldCheck } from "lucide-vue-next";
import { api, session, labels, date, type Batch, errorMessage } from "../api";
import { t, notificationText, pickupBy, number } from "../i18n";
const batches = ref<Batch[]>([]),
  notifications = ref<{ id: string; message: string; created_at: string }[]>(
    [],
  ),
  loading = ref(true),
  error = ref<unknown>(null);
onMounted(async () => {
  try {
    const r = await api<{
      batches: Batch[];
      notifications: typeof notifications.value;
    }>("/dashboard");
    batches.value = r.batches;
    notifications.value = r.notifications;
  } catch (e) {
    error.value = e;
  } finally {
    loading.value = false;
  }
});
</script>
<template>
  <section class="container page-section">
    <div class="page-heading split-heading">
      <div>
        <div class="eyebrow">{{ t("common.workspaceEyebrow") }}</div>
        <h1>
          {{
            session.user
              ? session.user.name + " · " + t("workspaceGreeting")
              : t("workspaceTitle")
          }}
        </h1>
        <p>{{ t("workspaceCopy") }}</p>
      </div>
      <RouterLink to="/share" class="button"
        >{{ t("shareButton") }}<ArrowUpRight :size="17"
      /></RouterLink>
    </div>
    <div v-if="loading" class="empty-state">{{ t("reading") }}</div>
    <template v-else-if="error"
      ><div class="empty-state" role="alert">
        <h3>{{ errorMessage(error) }}</h3>
        <RouterLink v-if="!session.user" to="/login" class="button">{{
          t("loginWorkspace")
        }}</RouterLink
        ><button v-else class="button" @click="$router.go(0)">
          {{ t("retry") }}
        </button>
      </div></template
    >
    <template v-else>
      <div class="org-grid">
        <div
          v-for="o in session.organizations"
          :key="o.id"
          class="surface org-card"
        >
          <ShieldCheck :size="25" />
          <div>
            <h3>{{ o.name }}</h3>
            <p>{{ labels[o.kind] }}</p>
          </div>
          <span class="status" :class="o.status">{{ labels[o.status] }}</span>
          <p v-if="o.status === 'pending'" class="org-note">
            {{ t("pendingOrg") }}
          </p>
        </div>
      </div>
      <div class="workspace-layout">
        <div>
          <h2 class="subheading">
            {{ t("foodJourney") }} <span>{{ batches.length }}</span>
          </h2>
          <div v-if="!batches.length" class="surface empty-state">
            <Leaf :size="34" />
            <h3>{{ t("startKindness") }}</h3>
            <p>{{ t("workspaceEmpty") }}</p>
            <RouterLink to="/" class="text-button"
              >{{ t("discover") }} <ArrowUpRight :size="16"
            /></RouterLink>
          </div>
          <div class="journey-list">
            <RouterLink
              v-for="b in batches"
              :key="b.id"
              :to="`/batches/${b.id}`"
              class="surface journey-card"
              ><div class="journey-icon"><Leaf :size="24" /></div>
              <div>
                <small
                  >{{ b.is_donor ? t("postedByMe") : t("claimedByMe") }} ·
                  {{ b.area }}</small
                >
                <h3>{{ b.title }}</h3>
                <p>
                  {{ number(b.quantity) }} {{ labels[b.unit] }} ·
                  {{ pickupBy(date(b.pickup_end)) }}
                </p>
                <p v-if="b.recipient_name" class="form-hint">
                  {{ t("recipientOrg") }}{{ b.recipient_name }}
                </p>
              </div>
              <div class="journey-end">
                <span class="status" :class="b.status">{{
                  labels[b.status]
                }}</span
                ><ArrowUpRight :size="18" /></div
            ></RouterLink>
          </div>
        </div>
        <aside class="surface notification-panel">
          <h3><Bell :size="18" />{{ t("messages") }}</h3>
          <p v-if="!notifications.length" class="muted">
            {{ t("noMessages") }}
          </p>
          <article v-for="n in notifications" :key="n.id">
            <p>{{ notificationText(n.message) }}</p>
            <small>{{ date(n.created_at) }}</small>
          </article>
        </aside>
      </div>
    </template>
  </section>
</template>
