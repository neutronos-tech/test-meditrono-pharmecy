import { useEffect, useState } from "react";
import { api, getToken, setToken } from "../api.js";
import { Phone } from "../Mock.jsx";
import { Eyebrow, Link, SITE, dark, light } from "../ui.jsx";

export const field = "mt-1 w-full rounded-2xl border border-line bg-cream px-4 py-3 outline-none focus:border-brand";

export function Auth({ mode, go }) {
  const signup = mode === "signup";
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [busy, setBusy] = useState(false);

  async function submit(event) {
    event.preventDefault();
    setError("");
    setBusy(true);
    try {
      const result = await (signup ? api.signup : api.login)(username.trim(), password);
      setToken(result.session_token);
      go("/app");
    } catch (failure) {
      if (failure.status === 409) setError("That username is taken. Try another.");
      else if (failure.status === 400) setError("Use 3–40 lowercase letters, numbers, - or _, and a password of at least 8 characters.");
      else if (failure.status === 401) setError("Wrong username or password.");
      else setError(failure.message);
    } finally {
      setBusy(false);
    }
  }

  return (
    <main className="grid gap-3 p-2 sm:p-3 md:grid-cols-2">
      <div className="hidden min-h-[640px] flex-col items-center justify-end overflow-hidden rounded-[2rem] bg-cream px-10 pt-16 md:flex">
        <p className="max-w-sm text-center text-3xl font-semibold tracking-tight">
          {signup ? "Your first month is on us." : "Welcome back to the counter."}
        </p>
        <Phone className="-mb-24 mt-12 w-[260px]" />
      </div>
      <form onSubmit={submit} className="flex flex-col justify-center px-4 py-12 sm:px-12">
        <Eyebrow>{signup ? "Create account" : "Sign in"}</Eyebrow>
        <h1 className="mt-6 text-4xl font-medium tracking-tight">{signup ? "Start your free month" : "Sign in"}</h1>
        <p className="mt-2 text-muted">{signup ? "Use the same username in the desktop app." : "Use your pharmacy username."}</p>
        <label className="mt-8 block text-sm font-medium">
          Pharmacy username
          <input className={field} value={username} onChange={(e) => setUsername(e.target.value)} autoComplete="username" required placeholder="rahman-pharmacy" />
        </label>
        <label className="mt-4 block text-sm font-medium">
          Password
          <input className={field} type="password" value={password} onChange={(e) => setPassword(e.target.value)} autoComplete={signup ? "new-password" : "current-password"} minLength={signup ? 8 : undefined} required />
        </label>
        {error && <p className="mt-4 rounded-2xl bg-red-50 p-3 text-sm text-danger">{error}</p>}
        <button disabled={busy} className={`${dark} mt-6 w-full py-3.5 disabled:opacity-60`}>
          {busy ? "Please wait…" : signup ? "Create account" : "Sign in"}
        </button>
        <p className="mt-6 text-sm text-muted">
          {signup ? "Already have an account? " : "New here? "}
          <Link to={signup ? "/login" : "/signup"} go={go} className="font-medium text-ink underline underline-offset-4">
            {signup ? "Sign in" : "Create an account"}
          </Link>
        </p>
      </form>
    </main>
  );
}

export function Dashboard({ go }) {
  const [account, setAccount] = useState(null);
  const [devices, setDevices] = useState(null);
  const [error, setError] = useState("");

  useEffect(() => {
    if (!getToken()) {
      go("/login");
      return;
    }
    api.me().then(setAccount).catch((failure) => (failure.status === 401 ? go("/login") : setError(failure.message)));
    api.devices().then(setDevices).catch(() => setDevices([]));
  }, []);

  async function signOut() {
    await api.logout().catch(() => {});
    setToken(null);
    go("/");
  }

  const now = Date.now();
  return (
    <main className="px-4 pb-16 pt-8 sm:px-8 lg:px-12">
      <div className="flex flex-wrap items-end justify-between gap-4">
        <div>
          <Eyebrow>Owner dashboard</Eyebrow>
          <h1 className="mt-4 text-4xl font-medium tracking-tight sm:text-6xl">{account?.username ?? "…"}</h1>
        </div>
        <button onClick={signOut} className={light}>Sign out</button>
      </div>
      {error && <p className="mt-4 rounded-2xl bg-red-50 p-3 text-sm text-danger">{error}</p>}

      <section className="mt-8 grid gap-3 md:grid-cols-2">
        <div className="min-h-[260px] rounded-[2rem] bg-ink">
          <div className="flex h-full flex-col justify-end p-6 text-white">
            <p className="text-2xl font-medium">Install on your shop computer</p>
            <p className="mt-2 max-w-sm text-sm text-white/70">Download, unzip and sign in with this username. Your first computer gets the free month.</p>
            <a href={SITE.downloadUrl} className={`${light} mt-4 self-start`}>Download for Windows</a>
          </div>
        </div>
        <div className="rounded-[2rem] bg-cream p-6">
          <p className="text-2xl font-medium">Payment</p>
          <p className="mt-2 text-sm text-muted">Online payment with bKash, Nagad and card is coming soon. To continue after your free month, contact us and we'll activate your computer.</p>
          <Link to="/contact" go={go} className={`${dark} mt-6`}>Contact us</Link>
        </div>
      </section>

      <section className="mt-10">
        <h2 className="text-2xl font-medium tracking-tight">Your computers</h2>
        <DeviceTable devices={devices} now={now} />
      </section>
    </main>
  );
}

export function DeviceTable({ devices, now, actions }) {
  return (
    <div className="mt-4 overflow-x-auto rounded-[2rem] border border-line">
      <table className="w-full min-w-[560px] text-left text-sm">
        <thead className="bg-cream text-xs uppercase tracking-wider text-muted">
          <tr>
            <th className="p-4 font-medium">Computer</th>
            <th className="p-4 font-medium">Status</th>
            <th className="p-4 font-medium">Paid until</th>
            <th className="p-4 font-medium">Last online</th>
            {actions && <th className="p-4" />}
          </tr>
        </thead>
        <tbody>
          {devices === null && <tr><td className="p-4 text-muted" colSpan="5">Loading…</td></tr>}
          {devices?.length === 0 && <tr><td className="p-4 text-muted" colSpan="5">No computer yet. Download the app and sign in to add one.</td></tr>}
          {devices?.map((device) => {
            const paid = new Date(device.paid_until);
            const ok = device.active && paid.getTime() > now;
            return (
              <tr key={device.id} className="border-t border-line">
                <td className="p-4 font-medium">{device.name} <span className="text-xs text-muted">#{device.id}</span></td>
                <td className="p-4">
                  <span className={`rounded-full px-2.5 py-1 text-xs font-medium ${ok ? "bg-brand-soft text-brand" : "bg-red-50 text-danger"}`}>
                    {!device.active ? "Deactivated" : ok ? "Active" : "Payment due"}
                  </span>
                </td>
                <td className="p-4">{paid.toLocaleDateString("en-GB")}</td>
                <td className="p-4 text-muted">{new Date(device.last_seen_at).toLocaleString("en-GB")}</td>
                {actions && <td className="p-4 text-right">{actions(device)}</td>}
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
