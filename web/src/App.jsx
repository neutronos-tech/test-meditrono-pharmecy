import { useState } from "react";
import { getToken } from "./api.js";
import { Link, SITE, Wordmark, dark, usePath } from "./ui.jsx";
import Home from "./pages/Home.jsx";
import { About, Contact, Features, Pricing } from "./pages/Marketing.jsx";
import { Auth, Dashboard } from "./pages/Account.jsx";
import Admin from "./pages/Admin.jsx";

const NAV = [
  ["/features", "Features"],
  ["/pricing", "Pricing"],
  ["/about", "MEDITRONO"],
  ["/contact", "Contact"],
];

export default function App() {
  const [path, go] = usePath();
  const pages = {
    "/": Home,
    "/features": Features,
    "/pricing": Pricing,
    "/about": About,
    "/contact": Contact,
    "/login": (props) => <Auth mode="login" {...props} />,
    "/signup": (props) => <Auth mode="signup" {...props} />,
    "/app": Dashboard,
    "/admin": Admin,
  };
  const Page = pages[path.replace(/\/$/, "") || "/"] ?? NotFound;
  return (
    <div className="mx-auto max-w-[1400px] px-2 py-2 sm:px-3 sm:py-3">
      <div className="rounded-[32px] bg-white">
        <Nav go={go} path={path} />
        <Page go={go} />
      </div>
      <Footer go={go} />
    </div>
  );
}

function Nav({ go, path }) {
  const [open, setOpen] = useState(false);
  const signedIn = Boolean(getToken());
  return (
    <header className="relative flex items-center justify-between px-5 py-4 sm:px-10">
      <Link to="/" go={go} className="text-sm font-semibold tracking-tight">
        Meditrino<sup>®</sup>
        <span className="ml-2 hidden text-xs font-normal text-muted sm:inline">by MEDITRONO</span>
      </Link>
      <nav className="hidden items-center gap-8 text-[13px] md:flex">
        {NAV.map(([to, label]) => (
          <Link key={to} to={to} go={go} className={path === to ? "text-ink" : "text-muted hover:text-ink"}>
            {label}
          </Link>
        ))}
      </nav>
      <div className="flex items-center gap-2">
        {signedIn ? (
          <Link to="/app" go={go} className={dark}>Dashboard</Link>
        ) : (
          <>
            <Link to="/login" go={go} className="hidden px-3 text-[13px] sm:inline">Sign in</Link>
            <Link to="/signup" go={go} className={dark}>Start free ↗</Link>
          </>
        )}
        <button className="ml-1 rounded-full border border-black/10 px-3 py-2 text-xs md:hidden" onClick={() => setOpen(!open)} aria-expanded={open}>
          Menu
        </button>
      </div>
      {open && (
        <div className="absolute inset-x-3 top-16 z-20 rounded-3xl border border-black/10 bg-white p-4 shadow-xl md:hidden">
          {[...NAV, ["/login", "Sign in"]].map(([to, label]) => (
            <Link key={to} to={to} go={(to) => { setOpen(false); go(to); }} className="block rounded-2xl px-4 py-3 hover:bg-cream">
              {label}
            </Link>
          ))}
        </div>
      )}
    </header>
  );
}

function Footer({ go }) {
  const columns = [
    ["Product", [["/features", "Features"], ["/pricing", "Pricing"], ["/signup", "Start free"], ["/login", "Sign in"]]],
    ["Company", [["/about", "MEDITRONO"], ["/contact", "Contact"]]],
  ];
  return (
    <footer className="mt-3 rounded-[32px] bg-night px-5 pb-8 pt-12 text-white sm:px-10">
      <div className="flex flex-wrap items-end justify-between gap-6">
        <Wordmark className="text-[22vw] sm:text-[15vw] xl:text-[200px]" />
        <p className="text-sm text-white/60">A MEDITRONO product · {new Date().getFullYear()}</p>
      </div>
      <div className="mt-12 grid gap-10 border-t border-white/10 pt-10 sm:grid-cols-4">
        <div className="sm:col-span-2">
          <p className="max-w-sm text-sm text-white/60">
            Pharmacy software from MEDITRONO, the digital health infrastructure company of Bangladesh.
          </p>
          {SITE.contactEmail && (
            <a href={`mailto:${SITE.contactEmail}`} className="mt-6 inline-block text-2xl font-medium underline decoration-white/30 underline-offset-8 sm:text-3xl">
              {SITE.contactEmail}
            </a>
          )}
        </div>
        {columns.map(([title, links]) => (
          <div key={title}>
            <p className="text-[11px] uppercase tracking-[0.18em] text-white/40">{title}</p>
            <ul className="mt-4 space-y-2 text-sm">
              {links.map(([to, label]) => (
                <li key={to}><Link to={to} go={go} className="text-white/80 hover:text-white">{label}</Link></li>
              ))}
            </ul>
          </div>
        ))}
      </div>
      <div className="mt-12 flex flex-wrap justify-between gap-3 text-xs text-white/40">
        <span>© {new Date().getFullYear()} MEDITRONO. All rights reserved.</span>
        <button onClick={() => window.scrollTo({ top: 0, behavior: "smooth" })} className="rounded-full border border-white/20 px-3 py-1 hover:text-white">
          Back to top ↑
        </button>
      </div>
    </footer>
  );
}

function NotFound({ go }) {
  return (
    <main className="px-5 py-24 text-center sm:px-10">
      <p className="text-7xl font-semibold tracking-tighter">404</p>
      <p className="mt-4 text-muted">This page doesn't exist.</p>
      <Link to="/" go={go} className={`${dark} mt-8`}>Go home</Link>
    </main>
  );
}
