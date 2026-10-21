import { reactive } from "vue";
import { localeDate, localizedLabels, t } from "./i18n";
export interface Organization {
  id: string;
  name: string;
  kind: "donor" | "recipient";
  status: string;
}
export interface User {
  id: string;
  email: string;
  name: string;
  is_admin: boolean;
}
export interface Batch {
  id: string;
  title: string;
  category: string;
  quantity: number;
  unit: string;
  weight_kg: number;
  area: string;
  pickup_start: string;
  pickup_end: string;
  expires_at: string;
  status: string;
  donor_name: string;
  claim_id?: string;
  claim_status?: string;
  recipient_name?: string;
  is_donor?: boolean;
  anchor_status?: string;
}
export const session = reactive<{
  user: User | null;
  organizations: Organization[];
  loaded: boolean;
}>({ user: null, organizations: [], loaded: false });
export class ApiError extends Error {
  constructor(
    public code: string,
    public status: number,
  ) {
    super(code);
    this.name = "ApiError";
  }
}
export function errorMessage(error: unknown): string {
  if (error instanceof ApiError && error.code === "COPY_FAILED")
    return t("notice.copyFailed");
  const key =
    error instanceof ApiError ? `error.${error.code}` : "error.REQUEST_FAILED";
  const message = t(key);
  return message === key ? t("error.REQUEST_FAILED") : message;
}
export async function api<T = Record<string, unknown>>(
  path: string,
  options: RequestInit = {},
): Promise<T> {
  let response: Response;
  try {
    response = await fetch(`/api/v1${path}`, {
      ...options,
      credentials: "same-origin",
      headers: {
        "Content-Type": "application/json",
        "X-Requested-With": "RescueProof",
        ...options.headers,
      },
    });
  } catch {
    throw new ApiError("network", 0);
  }
  const body = await response.json().catch(() => null);
  if (!response.ok) {
    const code = body?.error?.code ?? "REQUEST_FAILED";
    throw new ApiError(code, response.status);
  }
  return body as T;
}
export async function refreshSession() {
  try {
    const d = await api<{ user: User; organizations: Organization[] }>(
      "/auth/me",
    );
    session.user = d.user;
    session.organizations = d.organizations;
  } catch (e) {
    if (e instanceof ApiError && e.status === 401) {
      session.user = null;
      session.organizations = [];
    } else {
      throw e;
    }
  } finally {
    session.loaded = true;
  }
}
export const post = <T = Record<string, unknown>>(
  path: string,
  body: unknown = {},
  key = crypto.randomUUID(),
) =>
  api<T>(path, {
    method: "POST",
    headers: { "Idempotency-Key": key },
    body: JSON.stringify(body),
  });
export const labels: Record<string, string> = localizedLabels;
export const date = (s: string) =>
  new Intl.DateTimeFormat(localeDate.value, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  }).format(new Date(s));
export function pickupCode(raw: string): string {
  try {
    const data = JSON.parse(raw);
    if (typeof data.token === "string") return data.token.trim();
  } catch {
    /* Manual 64-character tokens are also accepted. */
  }
  return raw.trim();
}
export function localDateTime(iso: string): string {
  const d = new Date(iso);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
}
