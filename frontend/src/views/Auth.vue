<script setup lang="ts">
import { computed, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import { Leaf, ArrowUpRight, ShieldCheck } from "lucide-vue-next";
import { post, refreshSession, errorMessage } from "../api";
import { t } from "../i18n";
const route = useRoute(),
  router = useRouter(),
  register = computed(() => route.path === "/register");
const form = ref({
    name: "",
    email: "",
    password: "",
    organization_name: "",
    organization_kind: "donor",
  }),
  error = ref<unknown>(null),
  busy = ref(false);
async function submit() {
  if (busy.value) return;
  busy.value = true;
  error.value = null;
  try {
    await post(
      register.value ? "/auth/register" : "/auth/login",
      register.value
        ? form.value
        : { email: form.value.email, password: form.value.password },
    );
    await refreshSession();
    await router.push("/workspace");
  } catch (e) {
    error.value = e;
  } finally {
    busy.value = false;
  }
}
</script>
<template>
  <section class="auth-shell container">
    <div class="auth-story">
      <span class="eyebrow">{{ t("authWelcome") }}</span>
      <h1>{{ t("authTitle") }}</h1>
      <p>{{ t("authCopy") }}</p>
      <Leaf class="auth-leaf" :size="180" :stroke-width="0.7" />
      <div class="auth-security">
        <ShieldCheck :size="20" />{{ t("authSecurity") }}
      </div>
    </div>
    <div class="auth-panel">
      <h2>{{ register ? t("authJoin") : t("authHello") }}</h2>
      <p>{{ register ? t("authJoinCopy") : t("authLoginCopy") }}</p>
      <form @submit.prevent="submit">
        <label v-if="register"
          >{{ t("yourName")
          }}<input
            v-model="form.name"
            autocomplete="name"
            maxlength="100"
            required
            :placeholder="t('namePlaceholder')" /></label
        ><label
          >{{ t("email")
          }}<input
            v-model="form.email"
            type="email"
            autocomplete="email"
            maxlength="254"
            required
            placeholder="you@organisation.org" /></label
        ><label
          >{{ t("password")
          }}<input
            v-model="form.password"
            type="password"
            :autocomplete="register ? 'new-password' : 'current-password'"
            :minlength="register ? 12 : 1"
            maxlength="128"
            required
            :placeholder="
              register ? t('common.passwordMin') : t('common.passwordInput')
            " /></label
        ><template v-if="register"
          ><label
            >{{ t("orgName")
            }}<input
              v-model="form.organization_name"
              minlength="2"
              maxlength="160"
              required
              :placeholder="t('orgName')"
          /></label>
          <fieldset>
            <legend>{{ t("participate") }}</legend>
            <div class="role-options">
              <label :class="{ active: form.organization_kind === 'donor' }"
                ><input
                  v-model="form.organization_kind"
                  type="radio"
                  value="donor"
                />{{ t("donorRole") }}</label
              ><label
                :class="{ active: form.organization_kind === 'recipient' }"
                ><input
                  v-model="form.organization_kind"
                  type="radio"
                  value="recipient"
                />{{ t("recipientRole") }}</label
              >
            </div>
          </fieldset>
          <p class="form-hint">{{ t("reviewNote") }}</p></template
        >
        <p v-if="error" class="error" role="alert">{{ errorMessage(error) }}</p>
        <button class="button full" :disabled="busy">
          {{
            busy
              ? t("working")
              : register
                ? t("createAccount")
                : t("loginWorkspace")
          }}<ArrowUpRight :size="18" />
        </button>
      </form>
      <p class="auth-switch">
        {{ register ? t("alreadyJoined") : t("firstTime")
        }}<RouterLink :to="register ? '/login' : '/register'">{{
          register ? t("goLogin") : t("createOrg")
        }}</RouterLink>
      </p>
    </div>
  </section>
</template>
