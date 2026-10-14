import { useEffect, useState } from "react";
import { api, getToken, setToken, taka } from "./api.js";

const DOWNLOAD_URL = "https://github.com/neutronos-tech/test-meditrono-pharmecy/releases/latest";

function usePath() {
  const [path, setPath] = useState(window.location.pathname);
  useEffect(() => {
    const onPop = () => setPath(window.location.pathname);
    window.addEventListener("popstate", onPop);
    return () => window.removeEventListener("popstate", onPop);
  }, []);
  const go = (to) => {
    window.history.pushState(null, "", to);
    setPath(to);
    window.scrollTo(0, 0);
  };
  return [path, go];
}

function Link({ to, go, className, children }) {
  return (
    <a
      href={to}
      className={className}
      onClick={(event) => {
        event.preventDefault();
        go(to);
      }}
    >
      {children}
    </a>
  );
}

const button = "inline-flex items-center justify-center rounded-xl px-5 py-3 font-semibold transition";
const primary = `${button} bg-brand text-white hover:bg-brand-dark`;
const secondary = `${button} border border-line bg-white text-ink hover:bg-cream`;

export default function App() {
  const [path, go] = usePath();
  let page;
  if (path === "/login") page = <Auth mode="login" go={go} />;
  else if (path === "/signup") page = <Auth mode="signup" go={go} />;
  else if (path.startsWith("/app")) page = <Dashboard go={go} />;
  else page = <Home go={go} />;
  return (
    <div className="min-h-screen p-2 sm:p-4">
      <div className="mx-auto min-h-[calc(100vh-1rem)] max-w-6xl overflow-hidden rounded-3xl bg-white sm:min-h-[calc(100vh-2rem)]">
        <Header go={go} />
        {page}
        <footer className="border-t border-line px-6 py-6 text-sm text-muted">
          © {new Date().getFullYear()} MEDITRONO · Meditrino Pharmacy
        </footer>
      </div>
    </div>
  );
}

function Header({ go }) {
  const signedIn = Boolean(getToken());
  return (
    <header className="flex items-center justify-between gap-4 border-b border-line px-4 py-4 sm:px-6">
      <Link to="/" go={go} className="flex items-center gap-2 font-bold">
        <img src="/icon.svg" alt="" className="h-8 w-8" />
        <span>Meditrino Pharmacy</span>
      </Link>
      <nav className="flex items-center gap-2 text-sm">
        <a href="/#pricing" className="hidden px-3 py-2 text-muted hover:text-ink sm:inline">Pricing</a>
        {signedIn ? (
          <Link to="/app" go={go} className={`${primary} px-4 py-2`}>Dashboard</Link>
        ) : (
          <>
            <Link to="/login" go={go} className="px-3 py-2 font-medium">Sign in</Link>
            <Link to="/signup" go={go} className={`${primary} px-4 py-2`}>Start free</Link>
          </>
        )}
      </nav>
    </header>
  );
}

const FEATURES = [
  ["Works without internet", "Sell, buy and check stock during load-shedding or a bad connection. Data syncs when the internet comes back."],
  ["Expiry alerts", "See what expires soon, sell it first by batch, and return it to the supplier before it becomes a loss."],
  ["Bangladesh medicine catalogue", "Thousands of brands and generics, kept up to date for you. No typing medicine names."],
  ["Purchases and suppliers", "Record invoices from each distributor and receive stock batch by batch."],
  ["Owner dashboard", "Today's sales, stock health and devices — from your phone, wherever you are."],
  ["Your data is safe", "Every sale is stored on your computer and backed up to the cloud."],
];

const COMING = ["Photo of a supplier challan → stock in one tap", "Shelf, box and fridge locations", "Baki khata (customer credit)", "Bangla interface", "Prescription to cart"];

function Home({ go }) {
  const [plans, setPlans] = useState(null);
  useEffect(() => {
    api.plans().then(setPlans).catch(() => setPlans([]));
  }, []);
  return (
    <main>
      <section className="grid items-center gap-10 px-6 py-14 sm:py-20 lg:grid-cols-2">
        <div>
          <p className="mb-4 inline-block rounded-full bg-gold-soft px-3 py-1 text-sm font-medium text-ink">Made for pharmacies in Bangladesh</p>
          <h1 className="text-4xl font-bold leading-tight sm:text-5xl">Run your pharmacy, even when the internet doesn't.</h1>
          <p className="mt-5 text-lg text-muted">Sales, stock, expiry and purchases in one simple app. It works offline on your shop computer and keeps a safe copy in the cloud.</p>
          <div className="mt-8 flex flex-wrap gap-3">
            <Link to="/signup" go={go} className={primary}>Start your free month</Link>
            <a href={DOWNLOAD_URL} className={secondary}>Download for Windows</a>
          </div>
        </div>
        <div className="rounded-3xl bg-cream p-6">
          <div className="grid grid-cols-2 gap-3">
            {[["Today's sales", "৳18,450"], ["Medicines in stock", "1,204"], ["Expiring in 30 days", "17"], ["Low stock", "9"]].map(([label, value]) => (
              <div key={label} className="rounded-2xl border border-line bg-white p-4">
                <p className="text-sm text-muted">{label}</p>
                <p className="mt-1 text-2xl font-bold">{value}</p>
              </div>
            ))}
          </div>
          <p className="mt-4 text-center text-sm text-muted">Example dashboard</p>
        </div>
      </section>

      <section className="bg-cream px-6 py-14">
        <h2 className="text-3xl font-bold">Everything a counter needs</h2>
        <div className="mt-8 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {FEATURES.map(([title, text]) => (
            <div key={title} className="rounded-2xl border border-line bg-white p-5">
              <h3 className="font-semibold">{title}</h3>
              <p className="mt-2 text-sm text-muted">{text}</p>
            </div>
          ))}
        </div>
        <div className="mt-8 rounded-2xl border border-gold bg-gold-soft p-5">
          <h3 className="font-semibold">Coming soon</h3>
          <ul className="mt-2 grid gap-1 text-sm sm:grid-cols-2">
            {COMING.map((item) => <li key={item}>• {item}</li>)}
          </ul>
        </div>
      </section>

      <section className="px-6 py-14">
        <h2 className="text-3xl font-bold">How it works</h2>
        <ol className="mt-8 grid gap-4 sm:grid-cols-3">
          {["Create your account here", "Download the app and sign in on your shop computer", "Sell offline — it renews itself every time you're online"].map((step, index) => (
            <li key={step} className="rounded-2xl bg-brand-soft p-5">
              <span className="text-sm font-bold text-brand">Step {index + 1}</span>
              <p className="mt-1 font-medium">{step}</p>
            </li>
          ))}
        </ol>
      </section>

      <section id="pricing" className="bg-cream px-6 py-14">
        <h2 className="text-3xl font-bold">Simple pricing, per computer</h2>
        <p className="mt-2 text-muted">First month free. No card needed. Pay with bKash, Nagad or card.</p>
        <div className="mt-8 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {plans === null && <p className="text-muted">Loading prices…</p>}
          {plans?.length === 0 && (
            <div className="rounded-2xl border border-line bg-white p-6">
              <p className="font-semibold">Pricing announced soon</p>
              <p className="mt-2 text-sm text-muted">Join the free pilot today.</p>
            </div>
          )}
          {plans?.map((plan) => (
            <div key={plan.code} className="rounded-2xl border-2 border-brand bg-white p-6">
              <p className="font-semibold">{plan.name}</p>
              <p className="mt-3 text-3xl font-bold">
                {plan.monthly_price_paisa > 0 ? taka(plan.monthly_price_paisa) : "Free pilot"}
                {plan.monthly_price_paisa > 0 && <span className="text-base font-medium text-muted"> /month</span>}
              </p>
              {plan.yearly_price_paisa > 0 && <p className="text-sm text-muted">or {taka(plan.yearly_price_paisa)} a year</p>}
              <ul className="mt-4 space-y-1 text-sm">
                <li>✓ {plan.trial_days} days free</li>
                <li>✓ Works offline</li>
                <li>✓ {plan.ai_scan_cap} smart scans a month (soon)</li>
              </ul>
              <Link to="/signup" go={go} className={`${primary} mt-6 w-full`}>Start free</Link>
            </div>
          ))}
        </div>
      </section>
    </main>
  );
}

function Auth({ mode, go }) {
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

  const input = "mt-1 w-full rounded-xl border border-line bg-cream px-4 py-3 outline-none focus:border-brand";
  return (
    <main className="flex justify-center px-6 py-16">
      <form onSubmit={submit} className="w-full max-w-sm">
        <h1 className="text-3xl font-bold">{signup ? "Start your free month" : "Sign in"}</h1>
        <p className="mt-2 text-muted">{signup ? "Use the same login in the desktop app." : "Welcome back."}</p>
        <label className="mt-8 block text-sm font-medium">
          Pharmacy username
          <input className={input} value={username} onChange={(e) => setUsername(e.target.value)} autoComplete="username" required placeholder="rahman-pharmacy" />
        </label>
        <label className="mt-4 block text-sm font-medium">
          Password
          <input className={input} type="password" value={password} onChange={(e) => setPassword(e.target.value)} autoComplete={signup ? "new-password" : "current-password"} minLength={signup ? 8 : undefined} required />
        </label>
        {error && <p className="mt-4 rounded-xl bg-red-50 p-3 text-sm text-danger">{error}</p>}
        <button disabled={busy} className={`${primary} mt-6 w-full disabled:opacity-60`}>
          {busy ? "Please wait…" : signup ? "Create account" : "Sign in"}
        </button>
        <p className="mt-6 text-center text-sm text-muted">
          {signup ? "Already have an account? " : "New here? "}
          <Link to={signup ? "/login" : "/signup"} go={go} className="font-semibold text-brand">
            {signup ? "Sign in" : "Create an account"}
          </Link>
        </p>
      </form>
    </main>
  );
}

function Dashboard({ go }) {
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
    <main className="px-6 py-10">
      <div className="flex flex-wrap items-center justify-between gap-4">
        <div>
          <p className="text-sm text-muted">Signed in as</p>
          <h1 className="text-3xl font-bold">{account?.username ?? "…"}</h1>
        </div>
        <button onClick={signOut} className={secondary}>Sign out</button>
      </div>
      {error && <p className="mt-4 rounded-xl bg-red-50 p-3 text-sm text-danger">{error}</p>}

      <section className="mt-8 grid gap-4 sm:grid-cols-2">
        <div className="rounded-2xl bg-brand p-6 text-white">
          <h2 className="text-lg font-semibold">Install on your shop computer</h2>
          <p className="mt-2 text-sm text-brand-soft">Download, unzip and sign in with this same username. Your first computer gets the free month.</p>
          <a href={DOWNLOAD_URL} className={`${button} mt-4 bg-white text-brand hover:bg-cream`}>Download for Windows</a>
        </div>
        <div className="rounded-2xl bg-cream p-6">
          <h2 className="text-lg font-semibold">Payment</h2>
          <p className="mt-2 text-sm text-muted">Online payment with bKash, Nagad and card is coming soon. To continue after your free month, contact us and we'll activate your computer.</p>
        </div>
      </section>

      <section className="mt-8">
        <h2 className="text-xl font-bold">Your computers</h2>
        <div className="mt-4 overflow-x-auto rounded-2xl border border-line">
          <table className="w-full min-w-[480px] text-left text-sm">
            <thead className="bg-cream text-muted">
              <tr><th className="p-3">Computer</th><th className="p-3">Status</th><th className="p-3">Paid until</th><th className="p-3">Last online</th></tr>
            </thead>
            <tbody>
              {devices === null && <tr><td className="p-3 text-muted" colSpan="4">Loading…</td></tr>}
              {devices?.length === 0 && <tr><td className="p-3 text-muted" colSpan="4">No computer yet. Download the app and sign in to add one.</td></tr>}
              {devices?.map((device) => {
                const paid = new Date(device.paid_until);
                const ok = device.active && paid.getTime() > now;
                return (
                  <tr key={device.id} className="border-t border-line">
                    <td className="p-3 font-medium">{device.name}</td>
                    <td className="p-3">
                      <span className={`rounded-full px-2 py-1 text-xs font-semibold ${ok ? "bg-brand-soft text-brand" : "bg-red-50 text-danger"}`}>
                        {!device.active ? "Deactivated" : ok ? "Active" : "Payment due"}
                      </span>
                    </td>
                    <td className="p-3">{paid.toLocaleDateString("en-GB")}</td>
                    <td className="p-3">{new Date(device.last_seen_at).toLocaleString("en-GB")}</td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      </section>
    </main>
  );
}
