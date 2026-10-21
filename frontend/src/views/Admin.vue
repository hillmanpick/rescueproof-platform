<script setup lang="ts">
import { t } from "../i18n";
import { onMounted, ref } from "vue";
import { api, post, labels, type Organization, errorMessage } from "../api";
const orgs = ref<Organization[]>([]),
  disputes = ref<
    { id: string; title: string; reason: string; resolved_at: string | null }[]
  >([]),
  failed = ref<{ id: string; kind: string; attempts: number }[]>([]),
  error = ref<unknown>(null),
  busy = ref(false),
  note = ref(""),
  resolution = ref(""),
  selected = ref("");
async function load() {
  try {
    const d = await api<{
      organizations: Organization[];
      disputes: typeof disputes.value;
      failed_tasks: typeof failed.value;
    }>("/admin");
    orgs.value = d.organizations;
    disputes.value = d.disputes;
    failed.value = d.failed_tasks;
  } catch (e) {
    error.value = e;
  }
}
async function review(o: Organization, decision: string) {
  if (busy.value) return;
  busy.value = true;
  error.value = null;
  try {
    await post(`/admin/organizations/${o.id}/review`, {
      decision,
      note: note.value,
    });
    note.value = "";
    await load();
  } catch (e) {
    error.value = e;
  } finally {
    busy.value = false;
  }
}
async function resolve(id: string, outcome: string) {
  if (busy.value) return;
  busy.value = true;
  error.value = null;
  try {
    await post(`/admin/disputes/${id}/resolve`, {
      outcome,
      note: resolution.value,
    });
    selected.value = "";
    resolution.value = "";
    await load();
  } catch (e) {
    error.value = e;
  } finally {
    busy.value = false;
  }
}
onMounted(load);
</script>
<template>
  <section class="container page-section">
    <div class="page-heading">
      <div class="eyebrow">{{ t("common.adminEyebrow") }}</div>
      <h1>{{ t("admin.title") }}</h1>
      <p>{{ t("admin.description") }}</p>
    </div>
    <p v-if="error" class="error" role="alert">{{ errorMessage(error) }}</p>
    <div class="surface admin-panel">
      <h2>{{ t("admin.review") }}</h2>
      <label
        >{{ t("admin.note")
        }}<input
          v-model="note"
          :placeholder="t('admin.noteHint')"
          minlength="5"
          maxlength="1000"
      /></label>
      <div class="table-scroll">
        <table>
          <thead>
            <tr>
              <th>{{ t("admin.organization") }}</th>
              <th>{{ t("admin.kind") }}</th>
              <th>{{ t("admin.status") }}</th>
              <th>{{ t("admin.action") }}</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="o in orgs" :key="o.id">
              <td>{{ o.name }}</td>
              <td>{{ labels[o.kind] }}</td>
              <td>
                <span class="status" :class="o.status">{{
                  labels[o.status]
                }}</span>
              </td>
              <td>
                <button
                  v-if="o.status !== 'approved'"
                  class="button small"
                  :disabled="busy || note.trim().length < 5"
                  @click="review(o, 'approved')"
                >
                  {{ t("admin.approve") }}</button
                ><button
                  v-else
                  class="text-button danger"
                  :disabled="busy || note.trim().length < 5"
                  @click="review(o, 'suspended')"
                >
                  {{ t("admin.suspend") }}
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      <p v-if="!orgs.length" class="muted">{{ t("admin.noOrgs") }}</p>
    </div>
    <div class="surface admin-panel">
      <h2>{{ t("admin.disputes") }}</h2>
      <p v-if="!disputes.length" class="muted">{{ t("admin.noDisputes") }}</p>
      <article v-for="d in disputes" :key="d.id" class="dispute-item">
        <h3>{{ d.title }}</h3>
        <p>{{ d.reason }}</p>
        <span v-if="d.resolved_at" class="status completed">{{
          t("admin.resolved")
        }}</span
        ><template v-else
          ><button
            class="text-button"
            @click="selected = selected === d.id ? '' : d.id"
          >
            {{ t("admin.writeResolution") }}
          </button>
          <div v-if="selected === d.id">
            <label
              >{{ t("admin.resolution")
              }}<textarea
                v-model="resolution"
                minlength="5"
                maxlength="1000"
                required
              ></textarea>
            </label>
            <div class="button-row">
              <button
                class="button small"
                :disabled="busy || resolution.length < 5"
                @click="resolve(d.id, 'completed')"
              >
                {{ t("admin.restore") }}</button
              ><button
                class="button small secondary"
                :disabled="busy || resolution.length < 5"
                @click="resolve(d.id, 'cancelled')"
              >
                {{ t("admin.cancel") }}
              </button>
            </div>
            <p class="form-hint">{{ t("admin.resolveLimit") }}</p>
          </div></template
        >
      </article>
    </div>
    <div class="surface admin-panel">
      <h2>{{ t("admin.failed") }}</h2>
      <p v-if="!failed.length" class="muted">{{ t("admin.noFailed") }}</p>
      <p v-for="f in failed" :key="f.id">
        {{ f.kind }} · {{ t("admin.attempts", { count: f.attempts }) }} ·
        {{ f.id }}
      </p>
    </div>
  </section>
</template>
