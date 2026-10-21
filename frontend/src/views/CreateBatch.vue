<script setup lang="ts">
import { t } from "../i18n";
import { computed, ref } from "vue";
import { useRouter } from "vue-router";
import { ArrowUpRight, ShieldCheck, Leaf } from "lucide-vue-next";
import { session, post, localDateTime, errorMessage } from "../api";
const router = useRouter(),
  orgs = computed(() =>
    session.organizations.filter(
      (o) => o.kind === "donor" && o.status === "approved",
    ),
  );
const soon = new Date(Date.now() + 3600000),
  later = new Date(Date.now() + 10800000),
  expire = new Date(Date.now() + 14400000);
const form = ref({
  organization_id: "",
  title: "",
  category: "bakery",
  quantity: 1,
  unit: "portions",
  weight_kg: 1,
  area: "",
  pickup_address: "",
  contact: "",
  storage_notes: "",
  allergens: "",
  pickup_start: localDateTime(soon.toISOString()),
  pickup_end: localDateTime(later.toISOString()),
  expires_at: localDateTime(expire.toISOString()),
  safety_acknowledged: false,
});
const busy = ref(false),
  error = ref<unknown>(null);
let key = crypto.randomUUID();
async function submit() {
  if (busy.value) return;
  busy.value = true;
  error.value = null;
  try {
    const body = {
      ...form.value,
      organization_id: form.value.organization_id || orgs.value[0]?.id,
      pickup_start: new Date(form.value.pickup_start).toISOString(),
      pickup_end: new Date(form.value.pickup_end).toISOString(),
      expires_at: new Date(form.value.expires_at).toISOString(),
    };
    const r = await post<{ id: string }>("/batches", body, key);
    key = crypto.randomUUID();
    router.push(`/batches/${r.id}`);
  } catch (e) {
    error.value = e;
  } finally {
    busy.value = false;
  }
}
</script>
<template>
  <section class="container page-section">
    <div class="page-heading">
      <div class="eyebrow">{{ t("common.shareEyebrow") }}</div>
      <h1>{{ t("form.title") }}</h1>
      <p>{{ t("form.description") }}</p>
    </div>
    <div v-if="!session.loaded" class="empty-state">
      {{ t("form.checkAccount") }}
    </div>
    <div v-else-if="!session.user" class="empty-state">
      <Leaf :size="36" />
      <h3>{{ t("form.joinFirst") }}</h3>
      <p>{{ t("form.loginRequired") }}</p>
      <RouterLink class="button" to="/login">{{ t("goLogin") }}</RouterLink>
    </div>
    <div v-else-if="!orgs.length" class="empty-state">
      <ShieldCheck :size="36" />
      <h3>{{ t("form.approvalRequired") }}</h3>
      <p>{{ t("form.approvalHelp") }}</p>
      <RouterLink class="button" to="/workspace">{{
        t("form.orgStatus")
      }}</RouterLink>
    </div>
    <div v-else class="form-layout">
      <form class="surface batch-form" @submit.prevent="submit">
        <h2>{{ t("form.basics") }}</h2>
        <label v-if="orgs.length > 1"
          >{{ t("form.organization")
          }}<select v-model="form.organization_id" required>
            <option disabled value="">{{ t("form.chooseOrg") }}</option>
            <option v-for="o in orgs" :key="o.id" :value="o.id">
              {{ o.name }}
            </option>
          </select></label
        ><label
          >{{ t("form.name")
          }}<input
            v-model="form.title"
            required
            maxlength="160"
            :placeholder="t('form.nameExample')"
        /></label>
        <div class="form-grid">
          <label
            >{{ t("form.category")
            }}<select v-model="form.category">
              <option value="bakery">{{ t("label.bakery") }}</option>
              <option value="produce">{{ t("label.produce") }}</option>
              <option value="packaged">{{ t("label.packaged") }}</option>
              <option value="prepared">{{ t("label.prepared") }}</option>
            </select></label
          ><label
            >{{ t("form.area")
            }}<input
              v-model="form.area"
              required
              maxlength="100"
              :placeholder="t('form.areaExample')"
          /></label>
        </div>
        <div class="form-grid three">
          <label
            >{{ t("form.quantity")
            }}<input
              v-model.number="form.quantity"
              type="number"
              min="1"
              max="100000"
              required /></label
          ><label
            >{{ t("form.unit")
            }}<select v-model="form.unit">
              <option value="portions">{{ t("label.portions") }}</option>
              <option value="items">{{ t("label.items") }}</option>
              <option value="boxes">{{ t("label.boxes") }}</option>
            </select></label
          ><label
            >{{ t("form.weight")
            }}<input
              v-model.number="form.weight_kg"
              type="number"
              min="0.001"
              max="100000"
              step="0.001"
              required
          /></label>
        </div>
        <h2>{{ t("form.schedule") }}</h2>
        <div class="form-grid">
          <label
            >{{ t("form.start")
            }}<input
              v-model="form.pickup_start"
              type="datetime-local"
              required /></label
          ><label
            >{{ t("form.end")
            }}<input v-model="form.pickup_end" type="datetime-local" required
          /></label>
        </div>
        <label
          >{{ t("form.expires")
          }}<input v-model="form.expires_at" type="datetime-local" required
        /></label>
        <p class="form-hint">{{ t("form.timezone") }}</p>
        <label
          >{{ t("form.address")
          }}<input
            v-model="form.pickup_address"
            maxlength="500"
            required
            :placeholder="t('form.private')" /></label
        ><label
          >{{ t("form.contact")
          }}<input
            v-model="form.contact"
            maxlength="200"
            required
            :placeholder="t('form.private')"
        /></label>
        <h2>{{ t("form.safety") }}</h2>
        <label
          >{{ t("form.storage")
          }}<textarea
            v-model="form.storage_notes"
            maxlength="1000"
            required
            :placeholder="t('form.storageExample')"
          ></textarea></label
        ><label
          >{{ t("form.allergens")
          }}<textarea
            v-model="form.allergens"
            maxlength="500"
            required
            :placeholder="t('form.allergensExample')"
          ></textarea></label
        ><label class="checkbox"
          ><input
            v-model="form.safety_acknowledged"
            type="checkbox"
            required
          />{{ t("form.acknowledge") }}</label
        >
        <p v-if="error" class="error" role="alert">{{ errorMessage(error) }}</p>
        <button class="button" :disabled="busy">
          {{ busy ? t("form.publishing") : t("publish")
          }}<ArrowUpRight :size="17" />
        </button>
      </form>
      <aside class="form-aside">
        <ShieldCheck :size="32" />
        <h3>{{ t("form.safetyTitle") }}</h3>
        <p>{{ t("form.safetyCopy") }}</p>
        <ul>
          <li>{{ t("form.tipAllergens") }}</li>
          <li>{{ t("form.tipSafe") }}</li>
          <li>{{ t("form.tipTime") }}</li>
          <li>{{ t("form.tipQr") }}</li>
        </ul>
        <small>{{ t("form.safetyLimit") }}</small>
      </aside>
    </div>
  </section>
</template>
