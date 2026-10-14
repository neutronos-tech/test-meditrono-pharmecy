import { useState } from "react";
import { List, WhatsappLogo, X } from "@phosphor-icons/react";
import { getToken } from "./api.js";
import { BRAND, Link, Logo, SITE, Wordmark, dark, usePath } from "./ui.jsx";
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
    <div className="w-full">
      <Nav go={go} path={path} />
      <Page go={go} />
      <Footer go={go} />
      {SITE.whatsapp && (
        <a href={`https://wa.me/${SITE.whatsapp}`} className="fixed bottom-5 right-5 z-30 flex h-12 w-12 items-center justify-center rounded-full bg-brand text-white shadow-[0_12px_30px_-10px_rgba(12,33,24,0.5)]" aria-label="WhatsApp us">
          <WhatsappLogo size={24} weight="fill" />
        </a>
      )}
    </div>
  );
}

function Nav({ go, path }) {
  const [open, setOpen] = useState(false);
  const signedIn = Boolean(getToken());
  return (
    <header className="sticky top-0 z-20 border-b border-line/70 bg-white/80 backdrop-blur-xl">
      <div className="flex items-center justify-between px-4 py-3.5 sm:px-8 lg:px-12">
        <Link to="/" go={go} aria-label="M. Pharmacy home">
          <Logo />
        </Link>
        <nav className="hidden items-center gap-1 text-[13px] md:flex">
          {NAV.map(([to, label]) => (
            <Link key={to} to={to} go={go} className={`rounded-full px-4 py-2 transition ${path === to ? "bg-cream text-ink" : "text-muted hover:text-ink"}`}>
              {label}
            </Link>
          ))}
        </nav>
        <div className="flex items-center gap-2">
          {signedIn ? (
            <Link to="/app" go={go} className={dark}>Dashboard</Link>
          ) : (
            <>
              <Link to="/login" go={go} className="hidden px-3 text-[13px] text-muted hover:text-ink sm:inline">Sign in</Link>
              <Link to="/signup" go={go} className={dark}>Start free</Link>
            </>
          )}
          <button className="ml-1 rounded-full border border-line p-2 md:hidden" onClick={() => setOpen(!open)} aria-expanded={open} aria-label="Menu">
            {open ? <X size={18} /> : <List size={18} />}
          </button>
        </div>
      </div>
      {open && (
        <div className="border-t border-line bg-white p-3 md:hidden">
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
    <footer className="mt-24 overflow-hidden bg-ink px-4 pb-8 pt-16 text-white sm:px-8 lg:px-12">
      <div className="grid gap-10 md:grid-cols-[2fr_1fr_1fr]">
        <div>
          <Logo invert />
          <p className="mt-6 max-w-sm text-sm leading-relaxed text-white/60">
            {BRAND} is part of MEDITRONO, the digital health infrastructure company of Bangladesh.
          </p>
          {SITE.contactEmail && (
            <a href={`mailto:${SITE.contactEmail}`} className="mt-6 inline-block text-2xl font-medium hover:text-brand-light">
              {SITE.contactEmail}
            </a>
          )}
        </div>
        {columns.map(([title, links]) => (
          <div key={title}>
            <p className="text-xs uppercase tracking-[0.14em] text-white/40">{title}</p>
            <ul className="mt-4 space-y-2 text-sm">
              {links.map(([to, label]) => (
                <li key={to}><Link to={to} go={go} className="text-white/75 hover:text-brand-light">{label}</Link></li>
              ))}
            </ul>
          </div>
        ))}
      </div>
      <Wordmark className="mt-16 block whitespace-nowrap text-[13.5vw] text-white/[0.07]" />
      <div className="mt-6 flex flex-wrap justify-between gap-3 border-t border-white/10 pt-6 text-xs text-white/40">
        <span>© {new Date().getFullYear()} MEDITRONO. All rights reserved.</span>
      </div>
    </footer>
  );
}

function NotFound({ go }) {
  return (
    <main className="px-4 py-32 sm:px-8 lg:px-12">
      <p className="font-mono text-7xl tracking-tighter">404</p>
      <p className="mt-4 text-muted">This page doesn't exist.</p>
      <Link to="/" go={go} className={`${dark} mt-8`}>Go home</Link>
    </main>
  );
}
