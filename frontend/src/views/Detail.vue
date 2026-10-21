<script setup lang="ts">
import { t, number } from "../i18n";
import { computed, onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import QRCode from "qrcode";
import {
  MapPin,
  Clock,
  ShieldCheck,
  ArrowLeft,
  CheckCircle,
  Copy,
  AlertCircle,
} from "lucide-vue-next";
import {
  api,
  post,
  session,
  labels,
  date,
  pickupCode,
  type Batch,
  errorMessage,
  ApiError,
} from "../api";
const route = useRoute(),
  router = useRouter(),
  batch = ref<Batch | null>(null),
  privateInfo = ref<Record<string, string> | null>(null),
  journey = ref<Batch | null>(null),
  loading = ref(false),
  busy = ref(false),
  error = ref<unknown>(null),
  notice = ref<{ key: string; expires?: string } | null>(null),
  token = ref(""),
  qr = ref(""),
  entry = ref(""),
  weight = ref<number | undefined>(),
  reason = ref(""),
  events = ref<{ id: string; kind: string; created_at: string }[]>([]),
  proof = ref<Record<string, unknown>[]>([]),
  showDispute = ref(false);
const noticeMessage = computed(() =>
  notice.value
    ? t(
        notice.value.key,
        notice.value.expires ? { date: date(notice.value.expires) } : {},
      )
    : "",
);
const eligible = computed(() =>
  session.organizations.find(
    (o) => o.kind === "recipient" && o.status === "approved",
  ),
);
const cid = computed(() => journey.value?.claim_id);
const isDonor = computed(() => journey.value?.is_donor);
const eventLabels = computed<Record<string, string>>(() => ({
  "batch.created": t("event.batchCreated"),
  "batch.cancelled": t("event.batchCancelled"),
  "claim.created": t("event.claimCreated"),
  "claim.cancelled": t("event.claimCancelled"),
  "handover.donor_confirmed": t("event.donorConfirmed"),
  "handover.completed": t("event.received"),
  "dispute.opened": t("event.disputed"),
  "dispute.resolved": t("event.resolved"),
}));
async function load() {
  if (loading.value) return;
  loading.value = true;
  error.value = null;
  try {
    const r = await api<{
      batch: Batch;
      private: Record<string, string> | null;
    }>(`/batches/${route.params.id}`);
    batch.value = r.batch;
    privateInfo.value = r.private;
    const d = await api<{ batches: Batch[] }>("/dashboard");
    journey.value = d.batches.find((b) => b.id === route.params.id) || null;
    if (journey.value) {
      const [e, p] = await Promise.all([
        api<{ items: typeof events.value }>(
          `/batches/${route.params.id}/events`,
        ),
        api<{ items: typeof proof.value }>(`/batches/${route.params.id}/proof`),
      ]);
      events.value = e.items;
      proof.value = p.items;
    }
  } catch (e) {
    error.value = e;
  } finally {
    loading.value = false;
  }
}
onMounted(load);
async function action(path: string, body: unknown, success: string) {
  if (busy.value) return;
  busy.value = true;
  error.value = null;
  try {
    await post(path, body);
    notice.value = { key: success };
    token.value = "";
    qr.value = "";
    await load();
  } catch (e) {
    error.value = e;
  } finally {
    busy.value = false;
  }
}
async function claim() {
  if (!eligible.value) return;
  await action(
    `/batches/${route.params.id}/claims`,
    { organization_id: eligible.value.id },
    "notice.claimed",
  );
}
async function generate() {
  if (!cid.value || busy.value) return;
  busy.value = true;
  error.value = null;
  try {
    const r = await post<{ pickup_token: string; expires_at: string }>(
      `/claims/${cid.value}/pickup-token`,
    );
    token.value = r.pickup_token;
    qr.value = await QRCode.toDataURL(
      JSON.stringify({ claim_id: cid.value, token: token.value }),
      { margin: 1, width: 220, color: { dark: "#244b39", light: "#ffffff" } },
    );
    notice.value = { key: "notice.tokenExpiry", expires: r.expires_at };
  } catch (e) {
    error.value = e;
  } finally {
    busy.value = false;
  }
}
async function copy() {
  try {
    await navigator.clipboard.writeText(token.value);
    notice.value = { key: "notice.copied" };
  } catch {
    error.value = new ApiError("COPY_FAILED", 0);
  }
}
function downloadProof() {
  const blob = new Blob([JSON.stringify(proof.value, null, 2)], {
    type: "application/json",
  });
  const u = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = u;
  a.download = `rescueproof-${route.params.id}-proof.json`;
  a.click();
  URL.revokeObjectURL(u);
}
function transactionUrl(proof: Record<string, unknown>) {
  const explorers: Record<number, string> = {
    84532: "https://sepolia.basescan.org/tx/",
    11155111: "https://sepolia.etherscan.io/tx/",
  };
  const base = explorers[Number(proof.chain_id)];
  const hash = String(proof.transaction_hash || "");
  return base && /^0x[\da-f]{64}$/i.test(hash) ? `${base}${hash}` : "";
}
</script>
<template>
  <section class="container page-section">
    <button class="text-button back-link" @click="router.back()">
      <ArrowLeft :size="16" />{{ t("detail.back") }}
    </button>
    <div v-if="loading" class="empty-state">{{ t("detail.loading") }}</div>
    <template v-else
      ><p v-if="error" class="error" role="alert">
        {{ errorMessage(error) }}
        <RouterLink v-if="!session.user" to="/login">{{
          t("goLogin")
        }}</RouterLink
        ><button v-else @click="load">{{ t("detail.refresh") }}</button>
      </p>
      <p v-if="notice" class="success" role="status">{{ noticeMessage }}</p>
      <div v-if="batch" class="detail-layout">
        <div>
          <div class="surface detail-main">
            <div class="eyebrow">
              {{ batch.donor_name }} · {{ labels[batch.category] }}
            </div>
            <div class="detail-title">
              <h1>{{ batch.title }}</h1>
              <span class="status" :class="batch.status">{{
                labels[batch.status]
              }}</span>
            </div>
            <div class="detail-quantity">
              {{ number(batch.quantity) }}
              <span
                >{{ labels[batch.unit] }} {{ t("detail.estimate") }}
                {{ number(batch.weight_kg) }} kg</span
              >
            </div>
            <dl class="detail-facts">
              <div>
                <dt><MapPin :size="17" /> {{ t("form.area") }}</dt>
                <dd>{{ batch.area }}</dd>
              </div>
              <div>
                <dt><Clock :size="17" /> {{ t("detail.window") }}</dt>
                <dd>
                  {{ date(batch.pickup_start) }} — {{ date(batch.pickup_end) }}
                </dd>
              </div>
              <div>
                <dt>{{ t("detail.expires") }}</dt>
                <dd>{{ date(batch.expires_at) }}</dd>
              </div>
            </dl>
            <template v-if="privateInfo"
              ><h3>{{ t("detail.handoff") }}</h3>
              <dl class="detail-facts">
                <div>
                  <dt>{{ t("detail.address") }}</dt>
                  <dd>{{ privateInfo.pickup_address }}</dd>
                </div>
                <div>
                  <dt>{{ t("detail.contact") }}</dt>
                  <dd>{{ privateInfo.contact }}</dd>
                </div>
                <div>
                  <dt>{{ t("detail.storage") }}</dt>
                  <dd>{{ privateInfo.storage_notes }}</dd>
                </div>
                <div>
                  <dt>{{ t("detail.allergens") }}</dt>
                  <dd>{{ privateInfo.allergens }}</dd>
                </div>
              </dl></template
            >
            <p v-else class="info-note">
              <ShieldCheck :size="18" />{{ t("detail.privacy") }}
            </p>
          </div>
          <div v-if="events.length" class="surface timeline-panel">
            <h2>{{ t("detail.timeline") }}</h2>
            <div v-for="e in events" :key="e.id" class="timeline-item">
              <CheckCircle :size="18" />
              <div>
                <strong>{{ eventLabels[e.kind] || e.kind }}</strong
                ><small>{{ date(e.created_at) }}</small>
              </div>
            </div>
          </div>
          <div v-if="journey" class="surface proof-panel">
            <h3><ShieldCheck :size="20" />{{ t("detail.proof") }}</h3>
            <p v-if="!proof.length">{{ t("detail.pendingProof") }}</p>
            <template v-else
              ><div
                v-for="p in proof"
                :key="String(p.event_id)"
                class="proof-row"
              >
                <span>{{
                  p.status === "confirmed"
                    ? t("label.confirmed")
                    : p.status === "submitted"
                      ? t("detail.submittedProof")
                      : t("detail.batchedProof")
                }}</span
                ><code>{{ String(p.root).slice(0, 20) }}…</code>
                <a
                  v-if="transactionUrl(p)"
                  :href="transactionUrl(p)"
                  target="_blank"
                  rel="noopener noreferrer"
                  class="text-button proof-link"
                >{{ t("detail.viewTransaction") }}</a>
              </div>
              <button class="text-button" @click="downloadProof">
                {{ t("detail.download") }}
              </button></template
            >
            <p class="form-hint">{{ t("detail.proofLimit") }}</p>
          </div>
        </div>
        <aside class="surface action-panel">
          <h2>{{ t("detail.claimTitle") }}</h2>
          <p>{{ t("detail.claimCopy") }}</p>
          <template v-if="batch.status === 'open' && !journey"
            ><button
              v-if="eligible"
              class="button full"
              :disabled="busy"
              @click="claim"
            >
              {{ busy ? t("detail.working") : t("detail.claim") }}
            </button>
            <div v-else class="info-note">
              {{ t("detail.approvalRequired")
              }}<RouterLink to="/workspace">{{
                t("form.orgStatus")
              }}</RouterLink>
            </div></template
          ><template v-if="cid"
            ><template v-if="!isDonor && batch.status === 'claimed'"
              ><button class="button full" :disabled="busy" @click="generate">
                {{ busy ? t("detail.generating") : t("detail.generate") }}
              </button>
              <div v-if="qr" class="qr-panel">
                <img :src="qr" :alt="t('detail.qr')" /><code>{{ token }}</code
                ><button class="text-button" @click="copy">
                  <Copy :size="15" />{{ t("detail.copy") }}
                </button>
                <p>{{ t("detail.codePrivate") }}</p>
              </div>
              <button
                class="text-button danger"
                :disabled="busy"
                @click="action(`/claims/${cid}/cancel`, {}, 'notice.cancelled')"
              >
                {{ t("detail.cancelClaim") }}
              </button></template
            >
            <form
              v-if="isDonor && batch.status === 'claimed'"
              @submit.prevent="
                action(
                  `/claims/${cid}/handover`,
                  { pickup_token: pickupCode(entry) },
                  'notice.handedOver',
                )
              "
            >
              <label
                >{{ t("detail.inputCode")
                }}<textarea
                  v-model="entry"
                  required
                  :placeholder="t('detail.inputHint')"
                ></textarea></label
              ><button class="button full" :disabled="busy">
                {{ t("detail.confirmHandoff") }}
              </button>
            </form>
            <form
              v-if="!isDonor && batch.status === 'handed_over'"
              @submit.prevent="
                action(
                  `/claims/${cid}/receipt`,
                  { actual_weight_kg: weight ?? null },
                  'notice.received',
                )
              "
            >
              <label
                >{{ t("detail.receivedWeight")
                }}<input
                  v-model.number="weight"
                  type="number"
                  min="0"
                  max="100000"
                  step="0.001"
                  :placeholder="t('detail.unweighed')"
              /></label>
              <p class="form-hint">{{ t("detail.weightNote") }}</p>
              <button class="button full" :disabled="busy">
                {{ t("detail.confirmReceipt") }}
              </button>
            </form>
            <p
              v-if="isDonor && batch.status === 'handed_over'"
              class="info-note"
            >
              {{ t("detail.awaitReceipt") }}
            </p>
            <template
              v-if="
                ['claimed', 'handed_over', 'completed'].includes(batch.status)
              "
              ><button
                class="text-button danger"
                @click="showDispute = !showDispute"
              >
                <AlertCircle :size="16" />{{ t("detail.dispute") }}
              </button>
              <form
                v-if="showDispute"
                @submit.prevent="
                  action(
                    `/claims/${cid}/disputes`,
                    { reason },
                    'notice.disputed',
                  )
                "
              >
                <label
                  >{{ t("detail.reason")
                  }}<textarea
                    v-model="reason"
                    required
                    minlength="5"
                    maxlength="1000"
                    :placeholder="t('detail.reasonHint')"
                  ></textarea></label
                ><button class="button secondary full" :disabled="busy">
                  {{ t("detail.submitDispute") }}
                </button>
              </form></template
            ></template
          ><button
            v-if="isDonor && batch.status === 'open'"
            class="text-button danger"
            :disabled="busy"
            @click="
              action(`/batches/${batch.id}/cancel`, {}, 'notice.batchCancelled')
            "
          >
            {{ t("detail.cancelBatch") }}
          </button>
          <div class="action-safety">
            <ShieldCheck :size="24" />
            <p>{{ t("detail.safety") }}</p>
          </div>
        </aside>
      </div></template
    >
  </section>
</template>
