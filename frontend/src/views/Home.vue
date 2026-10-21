<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import {
  ArrowUpRight,
  ArrowRight,
  Leaf,
  HeartHandshake,
  Search,
  SlidersHorizontal,
  Sprout,
  PackageCheck,
  MapPin,
} from "lucide-vue-next";
import BatchCard from "../components/BatchCard.vue";
import { api, type Batch, errorMessage } from "../api";
import { t, number } from "../i18n";
const items = ref<Batch[]>([]),
  loading = ref(true),
  error = ref<unknown>(null),
  category = ref(""),
  search = ref("");
const impact = ref({
  completed_batches: 0,
  completed_weight_kg: 0,
  completed_claims: 0,
});
const filtered = computed(() =>
  items.value.filter(
    (b) =>
      (!category.value || b.category === category.value) &&
      `${b.title} ${b.area} ${b.donor_name}`
        .toLowerCase()
        .includes(search.value.toLowerCase()),
  ),
);
const categories = [
  ["", "all"],
  ["bakery", "bakery"],
  ["produce", "produce"],
  ["packaged", "packaged"],
  ["prepared", "prepared"],
];
async function load() {
  loading.value = true;
  error.value = null;
  try {
    const [b, i] = await Promise.all([
      api<{ items: Batch[] }>("/batches?limit=100"),
      api<typeof impact.value>("/impact"),
    ]);
    items.value = b.items;
    impact.value = i;
  } catch (e) {
    error.value = e;
  } finally {
    loading.value = false;
  }
}
onMounted(load);
</script>
<template>
  <section class="hero">
    <div class="container hero-grid">
      <div class="hero-copy">
        <div class="eyebrow">
          <span class="mini-dot"></span>{{ t("heroEyebrow") }}
        </div>
        <h1>
          {{ t("heroTitle") }}<br />{{ t("heroTitle2") }}
          <span class="hero-emphasis">{{ t("heroTitle3") }}</span>
        </h1>
        <p>{{ t("heroText") }}</p>
        <div class="hero-buttons">
          <a href="#discover" class="button"
            >{{ t("findFood") }} <ArrowUpRight :size="19" /></a
          ><RouterLink to="/share" class="text-button"
            >{{ t("shareFood") }} <ArrowRight :size="17"
          /></RouterLink>
        </div>
        <div class="hero-trust">
          <div class="trust-leaves">
            <Leaf :size="18" /><HeartHandshake :size="18" /><Sprout
              :size="18"
            />
          </div>
          <span>{{ t("trust") }}</span>
        </div>
      </div>
      <div class="hero-photo">
        <img
          class="hero-photo-image"
          src="/images/food-table-1280.webp"
          srcset="
            /images/food-table-640.webp   640w,
            /images/food-table-1280.webp 1280w
          "
          sizes="(max-width: 760px) calc(100vw - 36px), (max-width: 1240px) 52vw, 600px"
          width="1280"
          height="871"
          :alt="t('hero.photoAlt')"
          fetchpriority="high"
          decoding="async"
        />
        <div class="photo-caption">
          <span class="photo-caption-mark"><HeartHandshake :size="18" /></span>
          <div>
            <strong>{{ t("impactIntro") }}</strong
            ><small>RESCUEPROOF · {{ t("common.pilot") }}</small>
          </div>
        </div>
      </div>
    </div>
  </section>
  <section class="impact-strip">
    <div class="container impact-inner">
      <div class="impact-intro">
        <Sprout :size="27" /><span>{{ t("impactIntro") }}</span>
      </div>
      <div class="impact-number">
        <strong>{{ number(impact.completed_batches) }}</strong
        ><span>{{ t("completedBatches") }}</span>
      </div>
      <div class="impact-number">
        <strong>{{ number(impact.completed_weight_kg) }}<em>kg</em></strong
        ><span>{{ t("receivedWeight") }}</span>
      </div>
      <div class="impact-number">
        <strong>{{ number(impact.completed_claims) }}</strong
        ><span>{{ t("completedClaims") }}</span>
      </div>
      <div class="impact-disclaimer">{{ t("impactNote") }}</div>
    </div>
  </section>
  <section id="discover" class="section container">
    <div class="section-top">
      <div>
        <div class="eyebrow">{{ t("findGood") }}</div>
        <h2>{{ t("sectionTitle") }}</h2>
        <p>{{ t("sectionText") }}</p>
      </div>
      <div class="location-label"><MapPin :size="16" />{{ t("network") }}</div>
    </div>
    <div class="discovery-tools">
      <div class="chips">
        <button
          v-for="[value, label] in categories"
          :key="value"
          :class="{ selected: category === value }"
          @click="category = value"
        >
          {{ t(label) }}
        </button>
      </div>
      <label class="search-box"
        ><Search :size="17" /><input
          v-model="search"
          :placeholder="t('search')"
          :aria-label="t('search')" /><SlidersHorizontal :size="16"
      /></label>
    </div>
    <div v-if="loading" class="food-grid">
      <div v-for="n in 3" :key="n" class="skeleton"></div>
    </div>
    <div v-else-if="error" class="empty-state" role="alert">
      <Leaf :size="34" />
      <h3>{{ t("noConnection") }}</h3>
      <p>{{ errorMessage(error) }}</p>
      <button class="button small" @click="load">{{ t("retry") }}</button>
    </div>
    <div v-else-if="!filtered.length" class="empty-state">
      <Sprout :size="38" />
      <h3>{{ items.length ? t("noMatch") : t("firstShare") }}</h3>
      <p>{{ items.length ? t("searchAgain") : t("noBatch") }}</p>
      <RouterLink v-if="!items.length" to="/share" class="button small"
        >{{ t("publish") }} <ArrowUpRight :size="16"
      /></RouterLink>
    </div>
    <div v-else class="food-grid">
      <BatchCard v-for="batch in filtered" :key="batch.id" :batch="batch" />
    </div>
    <p class="section-footnote">{{ t("footnote") }}</p>
  </section>
  <section id="how" class="how-section">
    <div class="container">
      <div class="section-top">
        <div>
          <div class="eyebrow">{{ t("howEyebrow") }}</div>
          <h2>{{ t("howTitle") }}</h2>
        </div>
        <p>{{ t("howText") }}</p>
      </div>
      <div class="steps-grid">
        <article>
          <span class="step-icon"><Leaf :size="28" /></span
          ><span class="step-number">01</span>
          <h3>{{ t("step1") }}</h3>
          <p>{{ t("step1Text") }}</p>
        </article>
        <article>
          <span class="step-icon"><HeartHandshake :size="28" /></span
          ><span class="step-number">02</span>
          <h3>{{ t("step2") }}</h3>
          <p>{{ t("step2Text") }}</p>
        </article>
        <article>
          <span class="step-icon"><PackageCheck :size="28" /></span
          ><span class="step-number">03</span>
          <h3>{{ t("step3") }}</h3>
          <p>{{ t("step3Text") }}</p>
        </article>
      </div>
    </div>
  </section>
  <section class="container join-section">
    <div>
      <div class="eyebrow">{{ t("joinEyebrow") }}</div>
      <h2>{{ t("joinTitle") }}</h2>
      <p>{{ t("joinText") }}</p>
      <RouterLink to="/register" class="button"
        >{{ t("join") }} <ArrowUpRight :size="18"
      /></RouterLink>
    </div>
    <div class="join-emblem">
      <Sprout :size="100" :stroke-width="0.8" /><span>{{
        t("common.sharedFood")
      }}</span>
    </div>
  </section>
</template>
