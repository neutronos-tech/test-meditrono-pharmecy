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
    <>
      <Link to="/signup" go={go} className="block bg-brand px-4 py-2 text-center text-xs text-white">
        Free pilot in Naogaon: your first month is free. Join now →
      </Link>
      <div className="mx-auto max-w-[1360px]">
        <Nav go={go} path={path} />
        <Page go={go} />
        <Footer go={go} />
      </div>
      {SITE.whatsapp && (
        <a href={`https://wa.me/${SITE.whatsapp}`} className="fixed bottom-5 right-5 z-30 flex h-12 w-12 items-center justify-center rounded-full bg-brand text-xl text-white shadow-lg" aria-label="WhatsApp us">
          ✆
        </a>
      )}
    </>
  );
}

function Nav({ go, path }) {
  const [open, setOpen] = useState(false);
  const signedIn = Boolean(getToken());
  return (
    <header className="sticky top-0 z-20 flex items-center justify-between border-b border-line bg-white/90 px-5 py-4 backdrop-blur sm:px-10">
      <Link to="/" go={go} className="flex items-center gap-2 text-[15px] font-semibold tracking-tight">
        <img src="/icon.svg" alt="" className="h-6 w-6" />
        Meditrino<span className="text-brand">.</span>
      </Link>
      <nav className="hidden items-center gap-8 text-[13px] md:flex">
        {NAV.map(([to, label]) => (
          <Link key={to} to={to} go={go} className={path === to ? "font-medium text-ink" : "text-muted hover:text-ink"}>
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
            <Link to="/signup" go={go} className={dark}>Start free ↗</Link>
          </>
        )}
        <button className="ml-1 rounded-full border border-line px-3 py-2 text-xs md:hidden" onClick={() => setOpen(!open)} aria-expanded={open}>
          Menu
        </button>
      </div>
      {open && (
        <div className="absolute inset-x-3 top-full z-20 mt-2 rounded-3xl border border-line bg-white p-3 shadow-xl md:hidden">
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
    <footer className="mt-24 border-t border-line px-5 pb-8 pt-14 sm:px-10">
      <div className="grid gap-10 sm:grid-cols-4">
        <div className="sm:col-span-2">
          <p className="max-w-sm text-sm text-muted">
            Pharmacy software from MEDITRONO, the digital health infrastructure company of Bangladesh.
          </p>
          {SITE.contactEmail && (
            <a href={`mailto:${SITE.contactEmail}`} className="mt-6 inline-block text-2xl font-medium hover:text-brand">
              {SITE.contactEmail}
            </a>
          )}
        </div>
        {columns.map(([title, links]) => (
          <div key={title}>
            <p className="text-xs font-medium text-ink">{title}</p>
            <ul className="mt-4 space-y-2 text-sm">
              {links.map(([to, label]) => (
                <li key={to}><Link to={to} go={go} className="text-muted hover:text-ink">{label}</Link></li>
              ))}
            </ul>
          </div>
        ))}
      </div>
      <Wordmark className="mt-16 block text-[21vw] text-ink/[0.06] sm:text-[17vw] xl:text-[230px]" />
      <div className="mt-6 flex flex-wrap justify-between gap-3 border-t border-line pt-6 text-xs text-muted">
        <span>© {new Date().getFullYear()} MEDITRONO. All rights reserved.</span>
        <span>Photos: Unsplash</span>
      </div>
    </footer>
  );
}

function NotFound({ go }) {
  return (
    <main className="px-5 py-32 text-center sm:px-10">
      <p className="text-7xl font-semibold tracking-tighter">404</p>
      <p className="mt-4 text-muted">This page doesn't exist.</p>
      <Link to="/" go={go} className={`${dark} mt-8`}>Go home</Link>
    </main>
  );
}
