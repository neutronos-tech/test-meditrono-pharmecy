import { useState } from "react";
import { Arrow, Eyebrow, Link, PHOTOS, Photo, SITE, Tag, dark, light } from "../ui.jsx";

const SERVICES = [
  {
    title: "Offline counter",
    text: "Sell, check stock and print receipts with no internet. Load-shedding never stops the shop; everything syncs when you're back online.",
    tags: ["Point of sale", "Batches", "Receipts"],
    photo: PHOTOS.counsel,
  },
  {
    title: "Stock & expiry",
    text: "Every batch has its own expiry. The app sells the oldest first and warns you before medicine becomes a loss.",
    tags: ["Expiry alerts", "Low stock", "Returns"],
    photo: PHOTOS.shelves,
  },
  {
    title: "Purchases & suppliers",
    text: "Record each distributor's invoice and receive stock batch by batch. Soon: photograph the challan and the stock fills itself in.",
    tags: ["Invoices", "Distributors", "Smart intake (soon)"],
    photo: PHOTOS.inventory,
  },
  {
    title: "Owner dashboard",
    text: "Today's sales, stock health and every computer in your shop, from your phone, wherever you are.",
    tags: ["Web app", "iPhone & Android", "Reports"],
    photo: PHOTOS.pills,
  },
];

const FACTS = [
  ["30", "days of selling without connecting once"],
  ["7", "day read-only grace, never locked out"],
  ["৳0", "setup fee, first month free"],
  ["1", "computer is all you need"],
];

const PROCESS = [
  ["Sign up", "Create your pharmacy account on this website."],
  ["Install", "Download the app on your shop computer and sign in."],
  ["Sell offline", "Use it all day, with or without internet."],
  ["Renew online", "It renews itself whenever you're online."],
];

export default function Home({ go }) {
  const [open, setOpen] = useState(0);
  return (
    <main>
      {/* Hero */}
      <section className="px-3 pt-3 sm:px-5">
        <Photo src={PHOTOS.pharmacist} alt="A pharmacist taking a medicine box from a shelf" shade className="h-[78vh] min-h-[520px]">
          <div className="flex h-full flex-col justify-between p-5 text-white sm:p-10">
            <div className="flex flex-wrap gap-2">
              <Tag invert>Offline-first</Tag>
              <Tag invert>Made for Bangladesh</Tag>
              <Tag invert>by MEDITRONO</Tag>
            </div>
            <div className="flex flex-wrap items-end justify-between gap-6">
              <h1 className="max-w-3xl text-5xl font-medium leading-[0.98] tracking-tight sm:text-7xl lg:text-[96px]">
                The pharmacy counter, <span className="text-white/80">without the worry.</span>
              </h1>
              <div className="w-full max-w-xs rounded-[20px] bg-white p-5 text-ink">
                <p className="text-sm text-muted">Sales, stock and expiry that keep working when the internet doesn't.</p>
                <div className="mt-4 flex gap-2">
                  <Link to="/signup" go={go} className={dark}>Start free</Link>
                  <a href={SITE.downloadUrl} className={light}>Download</a>
                </div>
              </div>
            </div>
          </div>
        </Photo>
      </section>

      {/* Facts */}
      <section className="grid grid-cols-2 gap-y-10 border-b border-line px-5 py-16 sm:px-10 md:grid-cols-4">
        {FACTS.map(([value, label]) => (
          <div key={label} className="text-center">
            <p className="text-5xl font-medium tracking-tight sm:text-6xl">{value}</p>
            <p className="mx-auto mt-2 max-w-[180px] text-xs text-muted">{label}</p>
          </div>
        ))}
      </section>

      {/* Statement with photo */}
      <section className="grid items-center gap-10 px-5 py-24 sm:px-10 md:grid-cols-2">
        <Photo src={PHOTOS.counsel} alt="A pharmacist showing a medicine bottle to a customer" className="aspect-[4/5] md:aspect-square" />
        <div>
          <Eyebrow>Why Meditrino</Eyebrow>
          <h2 className="mt-6 text-4xl font-medium leading-[1.05] tracking-tight sm:text-6xl">Calm software for a busy counter.</h2>
          <p className="mt-6 max-w-md text-muted">
            Every sale, batch and expiry stays on your own computer, and a safe copy goes to the cloud whenever you're online. No server in the shop, no technician, no setup fee.
          </p>
          <div className="mt-8 flex flex-wrap gap-3">
            <Link to="/features" go={go} className={dark}>See features ↗</Link>
            <Link to="/about" go={go} className={light}>About MEDITRONO</Link>
          </div>
        </div>
      </section>

      {/* Bento */}
      <section className="grid gap-3 px-3 sm:px-5 md:grid-cols-3 md:grid-rows-2">
        <Photo src={PHOTOS.shelves} alt="Shelves stocked with medicine boxes" shade className="min-h-[380px] md:row-span-2">
          <div className="flex h-full flex-col justify-end p-6 text-white">
            <Tag invert>Stock</Tag>
            <p className="mt-3 text-3xl font-medium tracking-tight">Every batch, every expiry.</p>
          </div>
        </Photo>
        <div className="flex flex-col justify-between rounded-[24px] bg-cream p-6">
          <div className="flex justify-between">
            <Tag>Catalogue</Tag>
            <Arrow className="bg-ink text-white" />
          </div>
          <p className="mt-10 text-2xl font-medium tracking-tight">Bangladesh medicine catalogue, kept up to date for you.</p>
        </div>
        <Photo src={PHOTOS.inventory} alt="A woman checking stock with a tablet" shade className="min-h-[240px]">
          <div className="flex h-full flex-col justify-end p-6 text-white">
            <p className="text-2xl font-medium tracking-tight">Purchases in minutes.</p>
          </div>
        </Photo>
        <Photo src={PHOTOS.pills} alt="Medicine capsules" shade className="min-h-[240px]">
          <div className="flex h-full flex-col justify-end p-6 text-white">
            <p className="text-2xl font-medium tracking-tight">Smart intake. Soon.</p>
          </div>
        </Photo>
        <div className="flex flex-col justify-between rounded-[24px] bg-brand p-6 text-white">
          <Tag invert>Pilot</Tag>
          <div>
            <p className="text-2xl font-medium tracking-tight">Free pilot in Naogaon.</p>
            <Link to="/signup" go={go} className="mt-4 inline-flex items-center gap-2 text-sm">Join now <Arrow /></Link>
          </div>
        </div>
      </section>

      {/* Services */}
      <section className="px-5 py-24 sm:px-10">
        <div className="flex flex-wrap items-end justify-between gap-6">
          <div>
            <Eyebrow>What it does</Eyebrow>
            <h2 className="mt-6 max-w-xl text-4xl font-medium leading-[1.05] tracking-tight sm:text-6xl">Everything a counter needs.</h2>
          </div>
          <Link to="/features" go={go} className={light}>All features ↗</Link>
        </div>
        <div className="mt-12 border-t border-line">
          {SERVICES.map((service, index) => (
            <div key={service.title} className="border-b border-line py-6">
              <button className="grid w-full grid-cols-[40px_1fr_auto] items-center gap-4 text-left" onClick={() => setOpen(open === index ? -1 : index)} aria-expanded={open === index}>
                <span className="text-xs text-muted">0{index + 1}</span>
                <span className="text-xl font-medium sm:text-3xl">{service.title}</span>
                <span className={`flex h-9 w-9 items-center justify-center rounded-full border text-lg transition ${open === index ? "border-ink bg-ink text-white" : "border-line"}`}>{open === index ? "−" : "+"}</span>
              </button>
              {open === index && (
                <div className="mt-6 grid gap-6 sm:pl-14 md:grid-cols-2">
                  <div>
                    <p className="max-w-md text-muted">{service.text}</p>
                    <div className="mt-4 flex flex-wrap gap-2">{service.tags.map((t) => <Tag key={t}>{t}</Tag>)}</div>
                  </div>
                  <Photo src={service.photo} alt="" className="h-56" />
                </div>
              )}
            </div>
          ))}
        </div>
      </section>

      {/* Process */}
      <section className="mx-3 rounded-[28px] bg-cream px-5 py-20 sm:mx-5 sm:px-10">
        <Eyebrow>How it works</Eyebrow>
        <h2 className="mt-6 max-w-2xl text-4xl font-medium leading-[1.05] tracking-tight sm:text-6xl">From sign-up to selling in one afternoon.</h2>
        <div className="mt-12 grid gap-3 md:grid-cols-4">
          {PROCESS.map(([title, text], index) => (
            <div key={title} className="rounded-[20px] bg-white p-6">
              <span className="flex h-9 w-9 items-center justify-center rounded-full bg-brand-soft text-sm font-medium text-brand">{index + 1}</span>
              <p className="mt-10 text-lg font-medium">{title}</p>
              <p className="mt-1 text-sm text-muted">{text}</p>
            </div>
          ))}
        </div>
      </section>

      {/* CTA */}
      <section className="px-3 pt-24 sm:px-5">
        <Photo src={PHOTOS.sign} alt="A green pharmacy cross sign" shade className="h-[440px]">
          <div className="flex h-full flex-col items-center justify-center p-6 text-center text-white">
            <h2 className="max-w-2xl text-4xl font-medium leading-[1.05] tracking-tight sm:text-6xl">Your first month is on us.</h2>
            <p className="mt-4 text-white/80">No card. No setup fee. Cancel any time.</p>
            <div className="mt-8 flex flex-wrap justify-center gap-3">
              <Link to="/signup" go={go} className={`${light} border-white`}>Start free ↗</Link>
              <Link to="/pricing" go={go} className="inline-flex items-center rounded-full border border-white/40 px-5 py-2.5 text-sm text-white hover:bg-white/10">See pricing</Link>
            </div>
          </div>
        </Photo>
      </section>
    </main>
  );
}
