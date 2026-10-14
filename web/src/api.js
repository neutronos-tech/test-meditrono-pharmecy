// Talks to the Rust server. Set VITE_API_URL at build time; the server must
// list this site's origin in PHARMACY_WEB_ORIGINS for CORS.
export const API_URL = (import.meta.env.VITE_API_URL || "https://pharmacy.neutorium.com").replace(/\/$/, "");

const TOKEN_KEY = "meditrino.session";

export function getToken() {
  try {
    return localStorage.getItem(TOKEN_KEY);
  } catch {
    return null;
  }
}

export function setToken(token) {
  try {
    if (token) localStorage.setItem(TOKEN_KEY, token);
    else localStorage.removeItem(TOKEN_KEY);
  } catch {
    // Private mode: the session simply lasts for this tab.
  }
}

export class ApiError extends Error {
  constructor(status, message) {
    super(message);
    this.status = status;
  }
}

const ADMIN_KEY = "meditrino.admin";

export function getAdminToken() {
  try {
    return sessionStorage.getItem(ADMIN_KEY);
  } catch {
    return null;
  }
}

export function setAdminToken(token) {
  try {
    if (token) sessionStorage.setItem(ADMIN_KEY, token);
    else sessionStorage.removeItem(ADMIN_KEY);
  } catch {
    // Not persisted; the founder signs in again next time.
  }
}

async function request(path, { method = "GET", body, auth = false, admin = false } = {}) {
  const headers = {};
  if (body) headers["Content-Type"] = "application/json";
  if (auth) headers.Authorization = `Bearer ${getToken()}`;
  if (admin) headers.Authorization = `Bearer ${getAdminToken()}`;
  let response;
  try {
    response = await fetch(`${API_URL}${path}`, { method, headers, body: body && JSON.stringify(body) });
  } catch {
    throw new ApiError(0, "Cannot reach the server. Check your internet connection.");
  }
  if (!response.ok) {
    if (response.status === 401 && auth) setToken(null);
    throw new ApiError(response.status, (await response.text()) || "Request failed.");
  }
  return response.status === 204 ? null : response.json();
}

export const api = {
  plans: () => request("/api/v1/plans"),
  signup: (username, password) => request("/api/v1/auth/signup", { method: "POST", body: { username, password } }),
  login: (username, password) => request("/api/v1/auth/login", { method: "POST", body: { username, password } }),
  me: () => request("/api/v1/auth/me", { auth: true }),
  devices: () => request("/api/v1/devices", { auth: true }),
  logout: () => request("/api/v1/auth/logout", { method: "POST", auth: true }),
};

export const adminApi = {
  plans: () => request("/api/v1/admin/plans", { admin: true }),
  savePlan: (plan) => request(`/api/v1/admin/plans/${encodeURIComponent(plan.code)}`, { method: "PUT", body: plan, admin: true }),
  customers: (search, after) => {
    const query = new URLSearchParams();
    if (search) query.set("search", search);
    if (after) query.set("after", after);
    return request(`/api/v1/admin/customers?${query}`, { admin: true });
  },
  devices: (pharmacyId) => request(`/api/v1/admin/customers/${pharmacyId}/devices`, { admin: true }),
  extend: (deviceId, days) => request(`/api/v1/admin/devices/${deviceId}/extend`, { method: "POST", body: { days }, admin: true }),
  setActive: (deviceId, active) => request(`/api/v1/admin/devices/${deviceId}/active`, { method: "POST", body: { active }, admin: true }),
  audit: (before) => request(`/api/v1/admin/audit${before ? `?before=${before}` : ""}`, { admin: true }),
};

export function taka(paisa) {
  return `৳${(paisa / 100).toLocaleString("en-BD", { maximumFractionDigits: 0 })}`;
}
