import { useState } from "react";
import { ArrowUpRight, List, WhatsappLogo, X } from "@phosphor-icons/react";
import { getToken } from "./api.js";
import { useSmoothScroll, Reveal, Words } from "./motion.jsx";
import { IMAGES, Link, Logo, SITE, WRAP, dark, usePath } from "./ui.jsx";
import Home from "./pages/Home.jsx";
import { Contact, Features, Pricing } from "./pages/Marketing.jsx";
import { Auth, Dashboard } from "./pages/Account.jsx";
import Admin from "./pages/Admin.jsx";

const NAV = [
  ["/features", "Features"],
  ["/pricing", "Pricing"],
  ["/contact", "Contact"],
];

export default function App() {
  const [path, go] = usePath();
  useSmoothScroll();
  const pages = {
    "/": Home,
    "/features": Features,
    "/pricing": Pricing,
    "/contact": Contact,
    "/login": (props) => <Auth mode="login" {...props} />,
    "/signup": (props) => <Auth mode="signup" {...props} />,
    "/app": Dashboard,
    "/admin": Admin,
  };
  const Page = pages[path.replace(/\/$/, "") || "/"] ?? NotFound;
  return (
    <div className="w-full overflow-x-clip">
      <Nav go={go} path={path} />
      <Page key={path} go={go} />
      <Footer go={go} />
      {SITE.whatsapp && (
        <a href={`https://wa.me/${SITE.whatsapp}`} className="fixed bottom-5 right-5 z-30 flex h-12 w-12 items-center justify-center rounded-full bg-brand text-white shadow-[0_12px_30px_-10px_rgba(22,53,42,0.5)]" aria-label="WhatsApp us">
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
    <header className="sticky top-0 z-20 border-b border-line/70 bg-white/85 backdrop-blur-xl">
      <div className={`${WRAP} grid grid-cols-[1fr_auto] items-center py-3.5 md:grid-cols-[1fr_auto_1fr]`}>
        <Link to="/" go={go} className="justify-self-start" aria-label="M. Pharmacy home">
          <Logo />
        </Link>
        <nav className="hidden items-center gap-1 text-[13px] md:flex">
          {NAV.map(([to, label]) => (
            <Link key={to} to={to} go={go} className={`rounded-full px-4 py-2 transition ${path === to ? "bg-cream text-brand-dark" : "text-muted hover:text-brand-dark"}`}>
              {label}
            </Link>
          ))}
        </nav>
        <div className="flex items-center justify-self-end gap-2">
          {signedIn ? (
            <Link to="/app" go={go} className={`${dark} bg-brand-dark`}>Dashboard</Link>
          ) : (
            <>
              <Link to="/login" go={go} className="hidden px-3 text-[13px] text-muted hover:text-brand-dark sm:inline">Sign in</Link>
              <Link to="/signup" go={go} className={`${dark} bg-brand-dark`}>Start free</Link>
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
    ["Explore", [["/features", "Features"], ["/pricing", "Pricing"], [SITE.downloadUrl, "Download for Windows", true]]],
    ["Account", [["/signup", "Start free"], ["/login", "Sign in"], ["/app", "Owner dashboard"]]],
    ["Help", [["/contact", "Contact us"], ["/contact", "Book a setup visit"]]],
  ];
  const linkClass = "text-white/70 transition hover:text-white";
  return (
    <footer className="mt-28 bg-brand-dark text-white">
      {/* Call to action */}
      <div className={`${WRAP} grid gap-10 border-b border-white/10 pb-16 pt-20 md:grid-cols-12 md:items-end`}>
        <div className="md:col-span-7">
          <Words as="h2" text="Let's set up your pharmacy." accent="your pharmacy." className="max-w-[16ch] text-4xl font-semibold leading-[1.02] tracking-tighter md:text-6xl [&_.text-brand]:text-brand-light" />
          <Reveal>
            <p className="mt-6 max-w-[46ch] text-white/65">
              Tell us about your shop. We will help you load your stock and start selling, free for the first month.
            </p>
            <div className="mt-8 flex flex-wrap gap-3">
              <Link to="/signup" go={go} className="inline-flex items-center gap-2 rounded-full bg-white py-2 pl-5 pr-2 text-sm font-medium text-brand-dark transition active:scale-[0.98]">
                Start free
                <span className="flex h-8 w-8 items-center justify-center rounded-full bg-brand-dark text-white"><ArrowUpRight size={14} weight="bold" /></span>
              </Link>
              {SITE.whatsapp ? (
                <a href={`https://wa.me/${SITE.whatsapp}`} className="inline-flex items-center gap-2 rounded-full border border-white/25 px-5 py-2 text-sm transition hover:bg-white/10">
                  <WhatsappLogo size={18} /> WhatsApp
                </a>
              ) : (
                <Link to="/contact" go={go} className="inline-flex items-center rounded-full border border-white/25 px-5 py-2 text-sm transition hover:bg-white/10">Contact us</Link>
              )}
            </div>
          </Reveal>
        </div>
        <div className="md:col-span-4 md:col-start-9">
          <img src={IMAGES.sign} alt="The M. Pharmacy sign" loading="lazy" className="aspect-[4/3] w-full max-w-[460px] rounded-[1.75rem] object-cover md:ml-auto" />
        </div>
      </div>

      {/* Links */}
      <div className={`${WRAP} grid grid-cols-2 gap-10 py-14 md:grid-cols-12`}>
        {columns.map(([title, links]) => (
          <div key={title} className="md:col-span-3">
            <p className="text-sm font-medium">{title}</p>
            <ul className="mt-5 space-y-3 text-sm">
              {links.map(([to, label, external]) => (
                <li key={label}>
                  {external ? <a href={to} className={linkClass}>{label}</a> : <Link to={to} go={go} className={linkClass}>{label}</Link>}
                </li>
              ))}
            </ul>
          </div>
        ))}
        <div className="md:col-span-3">
          <p className="text-sm font-medium">Contact</p>
          <ul className="mt-5 space-y-3 text-sm text-white/70">
            {SITE.contactPhone && <li><a href={`tel:${SITE.contactPhone}`} className={linkClass}>{SITE.contactPhone}</a></li>}
            {SITE.contactEmail && <li><a href={`mailto:${SITE.contactEmail}`} className={linkClass}>{SITE.contactEmail}</a></li>}
            <li>Naogaon, Bangladesh</li>
            <li>Pilot open now</li>
          </ul>
        </div>
      </div>

      {/* Wordmark */}
      <div className={`${WRAP} overflow-hidden`}>
        <p className="select-none whitespace-nowrap text-center text-[15.5vw] font-semibold leading-[0.92] tracking-[-0.065em] text-white pb-[0.04em]" aria-hidden="true">
          M. Pharmacy
        </p>
      </div>
      <div className={`${WRAP} mt-8 flex flex-wrap justify-between gap-3 border-t border-white/10 py-6 text-xs text-white/45`}>
        <span>© {new Date().getFullYear()} M. Pharmacy, part of MEDITRONO.</span>
        <span>Made for pharmacies in Bangladesh.</span>
      </div>
    </footer>
  );
}

function NotFound({ go }) {
  return (
    <main className={`${WRAP} py-32`}>
      <p className="text-7xl font-semibold tracking-tighter text-brand-dark">404</p>
      <p className="mt-4 text-muted">This page doesn't exist.</p>
      <Link to="/" go={go} className={`${dark} mt-8 bg-brand-dark`}>Go home</Link>
    </main>
  );
}
