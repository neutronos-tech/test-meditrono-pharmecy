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

async function request(path, { method = "GET", body, auth = false } = {}) {
  const headers = {};
  if (body) headers["Content-Type"] = "application/json";
  if (auth) headers.Authorization = `Bearer ${getToken()}`;
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

export function taka(paisa) {
  return `৳${(paisa / 100).toLocaleString("en-BD", { maximumFractionDigits: 0 })}`;
}
