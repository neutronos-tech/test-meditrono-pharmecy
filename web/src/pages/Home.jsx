import { useState } from "react";
import { Eyebrow, Landscape, Link, SITE, Tag, Wordmark, dark, light } from "../ui.jsx";

const SERVICES = [
  {
    title: "Offline counter",
    text: "Sell, check stock and print receipts with no internet. Load-shedding or a bad connection never stops the shop; everything syncs when you're back online.",
    tags: ["Point of sale", "Batches", "Receipts"],
  },
  {
    title: "Stock & expiry",
    text: "Every batch has its own expiry. The app sells the oldest first and warns you before medicine becomes a loss, so you can return it to the supplier in time.",
    tags: ["Expiry alerts", "Low stock", "Returns"],
  },
  {
    title: "Purchases & suppliers",
    text: "Record each distributor's invoice and receive stock batch by batch. Soon: photograph the challan and the stock fills itself in.",
    tags: ["Invoices", "Distributors", "Smart intake (soon)"],
  },
  {
    title: "Owner dashboard",
    text: "Today's sales, stock health and every computer in your shop, from your phone, wherever you are.",
    tags: ["Web app", "iPhone & Android", "Reports"],
  },
];

const PROJECTS = [
  { scene: "meadow", title: "The counter that never goes offline", tags: ["Desktop app", "Windows"], note: "Works through load-shedding" },
  { scene: "dusk", title: "Your shop, from your phone", tags: ["Web dashboard", "PWA"], note: "Install from the browser" },
  { scene: "gold", title: "Challan in, stock done", tags: ["Smart intake", "Coming soon"], note: "Photo → stock in one tap" },
];

const PROCESS = [
  ["Sign up", "Create your pharmacy account on this website. It takes a minute."],
  ["Install", "Download the app on your shop computer and sign in."],
  ["Sell offline", "Use it all day, with or without internet."],
  ["Renew online", "It renews itself whenever you're online. Pay monthly per computer."],
];

const PAINS = [
  ["Losing money on expired medicine.", "Expiry alerts and expiry-first selling."],
  ["The internet goes, the shop stops.", "Works fully offline, syncs later."],
  ["Not seeing the shop when you're away.", "Owner dashboard on your phone."],
];

export default function Home({ go }) {
  const [open, setOpen] = useState(0);
  return (
    <main>
      {/* Hero */}
      <section className="px-5 sm:px-10">
        <div className="flex flex-wrap items-end justify-between gap-6 pt-6">
          <Wordmark className="text-[24vw] sm:text-[17vw] xl:text-[230px]" />
          <div className="max-w-xs pb-4">
            <div className="flex items-center gap-2">
              <div className="flex -space-x-2">
                {["#2f8a63", "#d8bb7e", "#7fbf95"].map((c) => (
                  <span key={c} className="h-8 w-8 rounded-full border-2 border-white" style={{ background: c }} />
                ))}
              </div>
              <Link to="/signup" go={go} className={`${dark} py-2 text-xs`}>Start free ↗</Link>
            </div>
            <p className="mt-4 text-sm text-muted">
              Pharmacy software for Bangladesh, from MEDITRONO. Sales, stock and expiry that keep working when the internet doesn't.
            </p>
          </div>
        </div>
        <Landscape scene="meadow" className="mt-4 h-[420px] sm:h-[560px]">
          <div className="flex h-full flex-col justify-between p-5 sm:p-8">
            <p className="max-w-[220px] text-xs text-ink/70">Offline-first. Synced to the cloud. Made for the counter.</p>
            <div className="flex flex-wrap items-end justify-between gap-4">
              <div className="flex flex-wrap gap-2 text-xs text-ink/80">
                {["Naogaon", "Rajshahi", "Bogura", "Dhaka"].map((city) => (
                  <span key={city} className="rounded-full bg-white/70 px-3 py-1 backdrop-blur">{city}</span>
                ))}
              </div>
              <div className="w-56 rounded-2xl bg-white/90 p-4 text-xs shadow-lg backdrop-blur">
                <p className="text-muted">Today's sales</p>
                <p className="mt-1 text-2xl font-semibold text-ink">৳18,450</p>
                <p className="mt-2 text-muted">17 batches expire this month</p>
                <p className="mt-2 text-[10px] text-muted">Example</p>
              </div>
            </div>
          </div>
        </Landscape>
      </section>

      {/* Statement */}
      <section className="grid gap-8 px-5 py-20 sm:px-10 md:grid-cols-[200px_1fr]">
        <Eyebrow>About the product</Eyebrow>
        <p className="text-3xl font-medium leading-[1.15] tracking-tight sm:text-[44px]">
          We believe a pharmacy should run on certainty, not on a good internet day.{" "}
          <span className="text-muted">
            Meditrino keeps every sale, batch and expiry on your own computer, and quietly keeps a safe copy in the cloud.
          </span>
        </p>
      </section>

      {/* Facts */}
      <section className="grid gap-3 px-5 sm:px-10 md:grid-cols-4">
        <div className="rounded-[24px] bg-cream p-6">
          <p className="text-sm text-muted">Pilot goal</p>
          <p className="mt-10 text-5xl font-semibold tracking-tight">10</p>
          <p className="mt-2 text-sm text-muted">pharmacies for our first free pilot, in Naogaon</p>
        </div>
        <div className="rounded-[24px] bg-night p-6 text-white">
          <p className="text-sm text-white/50">Offline</p>
          <p className="mt-10 text-5xl font-semibold tracking-tight">30<sup className="text-xl">days</sup></p>
          <p className="mt-2 text-sm text-white/60">of selling without connecting once</p>
        </div>
        <div className="rounded-[24px] border border-black/10 p-6">
          <p className="text-sm text-muted">Setup</p>
          <p className="mt-10 text-5xl font-semibold tracking-tight">৳0</p>
          <p className="mt-2 text-sm text-muted">and your first month is free</p>
        </div>
        <Landscape scene="mint" capsules={false} className="min-h-[220px]">
          <div className="flex h-full flex-col justify-end p-6">
            <p className="text-5xl font-semibold tracking-tight">24/7</p>
            <p className="mt-2 text-sm text-ink/70">your data, on your own computer</p>
          </div>
        </Landscape>
      </section>

      {/* Services */}
      <section className="mx-2 mt-20 rounded-[32px] bg-night px-5 py-14 text-white sm:mx-3 sm:px-10">
        <div className="flex flex-wrap items-end justify-between gap-6">
          <div>
            <Eyebrow invert>What it does</Eyebrow>
            <h2 className="mt-6 max-w-lg text-3xl font-medium leading-tight tracking-tight sm:text-5xl">
              Everything a counter needs, nothing it doesn't
            </h2>
          </div>
          <div className="flex flex-wrap gap-2">
            {["Offline", "Bangladesh catalogue", "Expiry"].map((t) => <Tag key={t} invert>{t}</Tag>)}
          </div>
        </div>
        <div className="mt-12 divide-y divide-white/10 border-t border-white/10">
          {SERVICES.map((service, index) => (
            <div key={service.title} className="py-6">
              <button className="grid w-full grid-cols-[40px_1fr_auto] items-center gap-4 text-left" onClick={() => setOpen(open === index ? -1 : index)} aria-expanded={open === index}>
                <span className="text-xs text-white/40">0{index + 1}</span>
                <span className="text-xl font-medium sm:text-2xl">{service.title}</span>
                <span className="flex h-9 w-9 items-center justify-center rounded-full border border-white/25 text-lg">{open === index ? "−" : "+"}</span>
              </button>
              {open === index && (
                <div className="mt-6 grid gap-6 pl-14 md:grid-cols-[1fr_1fr]">
                  <div>
                    <p className="max-w-md text-sm text-white/60">{service.text}</p>
                    <div className="mt-4 flex flex-wrap gap-2">{service.tags.map((t) => <Tag key={t} invert>{t}</Tag>)}</div>
                  </div>
                  <div className="grid grid-cols-2 gap-3">
                    <Landscape scene="meadow" className="h-32" />
                    <Landscape scene="dusk" capsules={false} className="h-32" />
                  </div>
                </div>
              )}
            </div>
          ))}
        </div>
        <Link to="/features" go={go} className={`${light} mt-8`}>See all features ↗</Link>
      </section>

      {/* Showcase */}
      <section className="space-y-3 px-2 pt-3 sm:px-3">
        {PROJECTS.map((project) => (
          <Landscape key={project.title} scene={project.scene} className="h-[380px] sm:h-[520px]">
            <div className={`flex h-full flex-col justify-between p-5 sm:p-8 ${project.scene === "dusk" ? "text-white" : "text-ink"}`}>
              <div className="flex justify-between text-xs opacity-70">
                <Eyebrow invert={project.scene === "dusk"}>Inside Meditrino</Eyebrow>
                <span>{project.note}</span>
              </div>
              <div>
                <h3 className="text-2xl font-medium tracking-tight sm:text-4xl">{project.title}</h3>
                <div className="mt-3 flex flex-wrap gap-2">{project.tags.map((t) => <Tag key={t} invert={project.scene === "dusk"}>{t}</Tag>)}</div>
              </div>
            </div>
          </Landscape>
        ))}
      </section>

      {/* Process */}
      <section className="px-5 py-20 sm:px-10">
        <div className="grid gap-6 md:grid-cols-[200px_1fr_1fr]">
          <Eyebrow>How it works</Eyebrow>
          <h2 className="text-3xl font-medium leading-tight tracking-tight sm:text-5xl">From sign-up to selling in one afternoon.</h2>
          <p className="text-sm text-muted">No server in the shop, no technician, no setup fee. If you have a Windows computer, you're ready.</p>
        </div>
        <div className="mt-12 grid gap-3 md:grid-cols-4">
          {PROCESS.map(([title, text], index) => (
            <div key={title} className="flex min-h-[260px] flex-col rounded-[20px] border border-black/10 p-4" style={{ paddingTop: `${16 + index * 28}px` }}>
              <span className="rounded-lg bg-night px-3 py-2 text-xs font-medium text-white">{title}</span>
              <div className="mt-auto">
                <p className="text-xs text-muted">0{index + 1}</p>
                <p className="mt-2 text-sm">{text}</p>
              </div>
            </div>
          ))}
        </div>
      </section>

      {/* Voices */}
      <section className="px-5 pb-20 sm:px-10">
        <div className="flex flex-wrap items-end justify-between gap-6">
          <div>
            <Eyebrow>Built from the counter</Eyebrow>
            <h2 className="mt-6 text-3xl font-medium tracking-tight sm:text-5xl">The problems we solve</h2>
          </div>
          <p className="max-w-sm text-sm text-muted">The problems pharmacy owners raise most, and what we built for each. Pilot stories from Naogaon are coming soon.</p>
        </div>
        <div className="mt-10 grid gap-3 md:grid-cols-[1fr_1fr_1fr_1fr]">
          <div className="flex flex-col justify-between rounded-[24px] bg-night p-6 text-white">
            <p className="text-sm text-white/50">Pilot</p>
            <p className="mt-6 text-4xl font-semibold">Naogaon</p>
            <p className="mt-2 text-sm text-white/60">First 10 pharmacies, free for a month.</p>
            <Link to="/signup" go={go} className={`${light} mt-6`}>Join the pilot ↗</Link>
          </div>
          {PAINS.map(([quote, answer]) => (
            <div key={quote} className="rounded-[24px] bg-cream p-6">
                            <p className="mt-2 text-lg font-medium leading-snug">{quote}</p>
              <p className="mt-6 border-t border-black/10 pt-4 text-sm text-muted">{answer}</p>
            </div>
          ))}
        </div>
        <div className="mt-10 flex flex-wrap gap-3">
          <Link to="/signup" go={go} className={dark}>Start your free month ↗</Link>
          <a href={SITE.downloadUrl} className={light}>Download for Windows</a>
        </div>
      </section>
    </main>
  );
}
