import { useEffect, useState } from "react";
import { adminApi, getAdminToken, setAdminToken, taka } from "../api.js";
import { Eyebrow, dark, light } from "../ui.jsx";
import { DeviceTable, field } from "./Account.jsx";

// Founder CMS. Protected by the server's PHARMACY_ADMIN_TOKEN; the token is
// kept only for this browser tab.
export default function Admin() {
  const [token, setToken] = useState(getAdminToken());
  const [tab, setTab] = useState("plans");
  if (!token) return <AdminLogin onDone={setToken} />;
  const signOut = () => {
    setAdminToken(null);
    setToken(null);
  };
  return (
    <main className="px-4 pb-16 pt-8 sm:px-8 lg:px-14">
      <div className="flex flex-wrap items-end justify-between gap-4">
        <div>
          <Eyebrow>Founder CMS</Eyebrow>
          <h1 className="mt-4 text-4xl font-medium tracking-tight sm:text-6xl">Control room</h1>
        </div>
        <button onClick={signOut} className={light}>Lock</button>
      </div>
      <div className="mt-8 flex flex-wrap gap-2">
        {[["plans", "Plans & prices"], ["customers", "Customers"], ["audit", "Audit log"]].map(([id, label]) => (
          <button key={id} onClick={() => setTab(id)} className={tab === id ? dark : light}>{label}</button>
        ))}
      </div>
      <div className="mt-8">
        {tab === "plans" && <Plans onUnauthorized={signOut} />}
        {tab === "customers" && <Customers onUnauthorized={signOut} />}
        {tab === "audit" && <Audit onUnauthorized={signOut} />}
      </div>
    </main>
  );
}

function AdminLogin({ onDone }) {
  const [value, setValue] = useState("");
  const [error, setError] = useState("");
  async function submit(event) {
    event.preventDefault();
    setAdminToken(value.trim());
    try {
      await adminApi.plans();
      onDone(value.trim());
    } catch (failure) {
      setAdminToken(null);
      setError(failure.status === 401 ? "That owner key is not right." : failure.status === 404 ? "The CMS is not switched on for this server yet." : failure.message);
    }
  }
  return (
    <main className="flex justify-center px-4 py-24">
      <form onSubmit={submit} className="w-full max-w-md">
        <Eyebrow>Founder CMS</Eyebrow>
        <h1 className="mt-6 text-4xl font-medium tracking-tight">Owner key</h1>
        <p className="mt-2 text-sm text-muted">Paste the PHARMACY_ADMIN_TOKEN from the server settings.</p>
        <input className={`${field} mt-6`} type="password" value={value} onChange={(e) => setValue(e.target.value)} autoComplete="off" required />
        {error && <p className="mt-4 rounded-2xl bg-red-50 p-3 text-sm text-danger">{error}</p>}
        <button className={`${dark} mt-6 w-full py-3.5`}>Unlock</button>
      </form>
    </main>
  );
}

function useLoad(load, onUnauthorized, deps = []) {
  const [data, setData] = useState(null);
  const [error, setError] = useState("");
  const reload = () =>
    load()
      .then((value) => {
        setData(value);
        setError("");
      })
      .catch((failure) => (failure.status === 401 ? onUnauthorized() : setError(failure.message)));
  useEffect(() => {
    reload();
  }, deps);
  return [data, error, reload, setData];
}

const toTaka = (paisa) => String(paisa / 100);
const toPaisa = (value) => Math.round(Number(value || 0) * 100);

function Plans({ onUnauthorized }) {
  const [plans, error, reload] = useLoad(adminApi.plans, onUnauthorized);
  const blank = { code: "", name: "", monthly_price_paisa: 0, yearly_price_paisa: 0, trial_days: 30, ai_scan_cap: 50, active: true, is_default: false };
  return (
    <div className="space-y-3">
      {error && <p className="rounded-2xl bg-red-50 p-3 text-sm text-danger">{error}</p>}
      {plans === null && <p className="text-muted">Loading…</p>}
      {plans?.map((plan) => <PlanEditor key={plan.code} plan={plan} onSaved={reload} />)}
      <details className="rounded-[2rem] border border-dashed border-black/20 p-6">
        <summary className="cursor-pointer font-medium">+ Add a plan</summary>
        <div className="mt-4"><PlanEditor plan={blank} isNew onSaved={reload} /></div>
      </details>
    </div>
  );
}

function PlanEditor({ plan, isNew, onSaved }) {
  const [draft, setDraft] = useState({ ...plan, monthly: toTaka(plan.monthly_price_paisa), yearly: toTaka(plan.yearly_price_paisa) });
  const [status, setStatus] = useState("");
  const set = (key) => (event) => setDraft({ ...draft, [key]: event.target.type === "checkbox" ? event.target.checked : event.target.value });
  async function save(event) {
    event.preventDefault();
    setStatus("Saving…");
    try {
      await adminApi.savePlan({
        code: draft.code.trim().toLowerCase(),
        name: draft.name,
        monthly_price_paisa: toPaisa(draft.monthly),
        yearly_price_paisa: toPaisa(draft.yearly),
        trial_days: Number(draft.trial_days),
        ai_scan_cap: Number(draft.ai_scan_cap),
        active: draft.active,
        is_default: draft.is_default,
      });
      setStatus("Saved. The website shows it now.");
      onSaved();
    } catch (failure) {
      setStatus(failure.status === 400 ? "Check the values. One active default plan must always exist." : failure.message);
    }
  }
  const input = "mt-1 w-full rounded-xl border border-line bg-cream px-3 py-2";
  return (
    <form onSubmit={save} className={`rounded-[2rem] p-6 ${isNew ? "" : "bg-cream/60 border border-line"}`}>
      <div className="flex flex-wrap items-center justify-between gap-2">
        <p className="text-lg font-medium">{isNew ? "New plan" : `${plan.name} · ${taka(plan.monthly_price_paisa)}/month`}</p>
        {plan.is_default && <span className="rounded-full bg-brand-soft px-3 py-1 text-xs text-brand">Default for new computers</span>}
      </div>
      <div className="mt-4 grid gap-3 sm:grid-cols-3 lg:grid-cols-6">
        <label className="text-xs text-muted">Code<input className={input} value={draft.code} onChange={set("code")} disabled={!isNew} required pattern="[a-z0-9-]{2,40}" /></label>
        <label className="text-xs text-muted">Name<input className={input} value={draft.name} onChange={set("name")} required /></label>
        <label className="text-xs text-muted">Monthly (৳)<input className={input} type="number" min="0" step="1" value={draft.monthly} onChange={set("monthly")} /></label>
        <label className="text-xs text-muted">Yearly (৳)<input className={input} type="number" min="0" step="1" value={draft.yearly} onChange={set("yearly")} /></label>
        <label className="text-xs text-muted">Trial days<input className={input} type="number" min="0" max="365" value={draft.trial_days} onChange={set("trial_days")} /></label>
        <label className="text-xs text-muted">AI scans/month<input className={input} type="number" min="0" value={draft.ai_scan_cap} onChange={set("ai_scan_cap")} /></label>
      </div>
      <div className="mt-4 flex flex-wrap items-center gap-5 text-sm">
        <label className="flex items-center gap-2"><input type="checkbox" checked={draft.active} onChange={set("active")} /> Shown on website</label>
        <label className="flex items-center gap-2"><input type="checkbox" checked={draft.is_default} onChange={set("is_default")} /> Default plan</label>
        <button className={dark}>Save</button>
        {status && <span className="text-muted">{status}</span>}
      </div>
    </form>
  );
}

function Customers({ onUnauthorized }) {
  const [search, setSearch] = useState("");
  const [query, setQuery] = useState("");
  const [customers, error] = useLoad(() => adminApi.customers(query), onUnauthorized, [query]);
  const [openId, setOpenId] = useState(null);
  return (
    <div>
      <form onSubmit={(e) => { e.preventDefault(); setQuery(search.trim()); }} className="flex gap-2">
        <input className={`${field} mt-0`} placeholder="Search username" value={search} onChange={(e) => setSearch(e.target.value)} />
        <button className={dark}>Search</button>
      </form>
      {error && <p className="mt-4 rounded-2xl bg-red-50 p-3 text-sm text-danger">{error}</p>}
      <div className="mt-6 space-y-3">
        {customers === null && <p className="text-muted">Loading…</p>}
        {customers?.length === 0 && <p className="text-muted">No customers found.</p>}
        {customers?.map((customer) => (
          <div key={customer.id} className="rounded-[2rem] border border-line p-5">
            <button className="flex w-full flex-wrap items-center justify-between gap-3 text-left" onClick={() => setOpenId(openId === customer.id ? null : customer.id)}>
              <div>
                <p className="text-lg font-medium">{customer.username}</p>
                <p className="text-xs text-muted">Joined {new Date(customer.created_at).toLocaleDateString("en-GB")}</p>
              </div>
              <div className="flex items-center gap-3 text-sm">
                <span>{customer.paid_devices}/{customer.devices} computers active</span>
                <span className="flex h-8 w-8 items-center justify-center rounded-full border border-black/15">{openId === customer.id ? "−" : "+"}</span>
              </div>
            </button>
            {openId === customer.id && <CustomerDevices pharmacyId={customer.id} onUnauthorized={onUnauthorized} />}
          </div>
        ))}
      </div>
      <p className="mt-4 text-xs text-muted">Shows up to 100 customers; search to narrow down.</p>
    </div>
  );
}

function CustomerDevices({ pharmacyId, onUnauthorized }) {
  const [devices, error, reload] = useLoad(() => adminApi.devices(pharmacyId), onUnauthorized, [pharmacyId]);
  const [busy, setBusy] = useState(false);
  const run = async (action) => {
    setBusy(true);
    try {
      await action();
      await reload();
    } catch (failure) {
      if (failure.status === 401) onUnauthorized();
      else alert(failure.message);
    } finally {
      setBusy(false);
    }
  };
  const small = "rounded-full border border-line px-3 py-1 text-xs hover:bg-cream disabled:opacity-50";
  return (
    <div className="mt-4">
      {error && <p className="rounded-2xl bg-red-50 p-3 text-sm text-danger">{error}</p>}
      <DeviceTable
        devices={devices}
        now={Date.now()}
        actions={(device) => (
          <div className="flex justify-end gap-1">
            <button disabled={busy} className={small} onClick={() => run(() => adminApi.extend(device.id, 30))}>+30 days</button>
            <button disabled={busy} className={small} onClick={() => run(() => adminApi.extend(device.id, 365))}>+1 year</button>
            <button disabled={busy} className={small} onClick={() => run(() => adminApi.setActive(device.id, !device.active))}>
              {device.active ? "Deactivate" : "Reactivate"}
            </button>
          </div>
        )}
      />
    </div>
  );
}

function Audit({ onUnauthorized }) {
  const [entries, error] = useLoad(() => adminApi.audit(), onUnauthorized);
  return (
    <div className="overflow-x-auto rounded-[2rem] border border-line">
      {error && <p className="m-4 rounded-2xl bg-red-50 p-3 text-sm text-danger">{error}</p>}
      <table className="w-full min-w-[640px] text-left text-sm">
        <thead className="bg-cream text-xs uppercase tracking-wider text-muted">
          <tr><th className="p-4 font-medium">When</th><th className="p-4 font-medium">Action</th><th className="p-4 font-medium">By</th><th className="p-4 font-medium">Details</th></tr>
        </thead>
        <tbody>
          {entries === null && <tr><td className="p-4 text-muted" colSpan="4">Loading…</td></tr>}
          {entries?.map((entry) => (
            <tr key={entry.id} className="border-t border-line align-top">
              <td className="whitespace-nowrap p-4 text-muted">{new Date(entry.created_at).toLocaleString("en-GB")}</td>
              <td className="p-4 font-medium">{entry.action}</td>
              <td className="p-4">{entry.actor} <span className="text-xs text-muted">({entry.source})</span></td>
              <td className="p-4 font-mono text-xs text-muted">{JSON.stringify(entry.detail)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
