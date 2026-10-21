<script setup lang="ts">
import {
  MapPin,
  Clock,
  ArrowUpRight,
  Leaf,
  Wheat,
  Package,
  Utensils,
} from "lucide-vue-next";
import { labels, date, type Batch } from "../api";
import { t, pickupBy, number } from "../i18n";
defineProps<{ batch: Batch }>();
const icons = {
  bakery: Wheat,
  produce: Leaf,
  packaged: Package,
  prepared: Utensils,
};
</script>
<template>
  <RouterLink :to="`/batches/${batch.id}`" class="food-card"
    ><div class="food-visual" :class="batch.category">
      <span class="food-category"
        ><component
          :is="icons[batch.category as keyof typeof icons] || Leaf"
          :size="14"
        />{{ labels[batch.category] }}</span
      >
      <div class="food-art" aria-hidden="true">
        <component
          :is="icons[batch.category as keyof typeof icons] || Leaf"
          :size="58"
          :stroke-width="1.2"
        />
      </div>
      <span class="availability"
        ><span></span>{{ labels[batch.status] || batch.status }}</span
      >
    </div>
    <div class="food-body">
      <div class="food-donor">{{ batch.donor_name }}</div>
      <h3>{{ batch.title }}</h3>
      <p class="food-meta">
        <MapPin :size="14" />{{ batch.area }}<span class="food-dot">·</span
        >{{ number(batch.weight_kg) }} kg
      </p>
      <p class="food-meta">
        <Clock :size="14" />{{ pickupBy(date(batch.pickup_end)) }}
      </p>
      <div class="card-bottom">
        <strong
          >{{ number(batch.quantity) }}
          <span>{{ labels[batch.unit] }}</span></strong
        ><span class="card-link"
          >{{ t("viewDetails") }} <ArrowUpRight :size="16"
        /></span>
      </div></div
  ></RouterLink>
</template>
