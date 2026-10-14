import { ArrowUpRight, Barcode, CloudCheck, Package, WifiSlash } from "@phosphor-icons/react";
import { Laptop } from "../Mock.jsx";
import { Badge, Counter, ParallaxImg, Reveal, ScaleIn, Words } from "../motion.jsx";
import { Eyebrow, IMAGES, Link, SITE, WRAP } from "../ui.jsx";

const FACTS = [
  ["30", "days of selling with no internet"],
  ["7", "day grace, never locked out"],
  ["৳0", "setup fee, first month free"],
  ["1", "Windows computer is enough"],
];

const AREAS = [
  { image: IMAGES.scanner, label: "Counter", text: "Sell in seconds", alt: "Scanning a medicine box at the counter" },
  { image: IMAGES.shelves, label: "Stock", text: "Every batch tracked", alt: "Pharmacy shelves full of medicine" },
  { image: IMAGES.capsules, label: "Expiry", text: "Sell the oldest first", alt: "Green and white capsules" },
  { image: IMAGES.doctor, label: "Owner", text: "Your shop on your phone", alt: "A pharmacist checking a tablet" },
];

const STEPS = [
  ["Sign up", "Create your pharmacy account on this website."],
  ["Install", "Download the app on your shop computer and sign in."],
  ["Sell", "Use it all day, with or without internet."],
  ["Renew", "It renews itself whenever you are online."],
];

function ArrowButton({ dark = false }) {
  return (
    <span className={`flex h-10 w-10 items-center justify-center rounded-full transition duration-500 group-hover:rotate-45 ${dark ? "bg-brand-dark text-white" : "bg-white text-brand-dark"}`}>
      <ArrowUpRight size={16} weight="bold" />
    </span>
  );
}

export default function Home({ go }) {
  return (
    <main>
      {/* Hero */}
      <section className={`${WRAP} pt-12 lg:pt-16`}>
        <Reveal className="flex items-center justify-between">
          <Eyebrow>Pharmacy software for Bangladesh</Eyebrow>
          <span className="hidden text-xs text-muted sm:block">Free pilot open in Naogaon</span>
        </Reveal>
        <Words
          as="h1"
          text="Care at the counter, made simple."
          accent="made simple."
          className="mt-8 max-w-[15ch] text-[13vw] font-semibold leading-[0.95] tracking-[-0.05em] text-brand-dark sm:text-7xl lg:text-[112px]"
        />
        <div className="mt-12 grid gap-10 pb-20 md:grid-cols-12 md:gap-6">
          <Reveal className="md:col-span-4 md:pt-4" delay={0.3}>
            <p className="max-w-[40ch] text-lg leading-relaxed text-muted">
              Sales, stock and expiry on your shop computer. It keeps working without internet and keeps a safe copy in the cloud.
            </p>
            <div className="mt-8 flex flex-wrap items-center gap-5">
              <Link to="/signup" go={go} className="group inline-flex items-center gap-3 rounded-full bg-brand-dark py-2 pl-6 pr-2 text-[15px] font-medium text-white transition active:scale-[0.98]">
                Start your free month <ArrowButton />
              </Link>
              <a href={SITE.downloadUrl} className="text-[15px] text-brand underline-offset-4 hover:underline">Download for Windows</a>
            </div>
          </Reveal>
          <div className="relative grid grid-cols-2 gap-3 md:col-span-8 md:col-start-5 md:grid-cols-[1.1fr_1fr]">
            <ParallaxImg src={IMAGES.shelves} alt="A bright pharmacy with stocked shelves" className="row-span-2 aspect-[3/4] md:aspect-auto md:h-full" />
            <ParallaxImg src={IMAGES.capsules} alt="Green and white capsules on a counter" className="aspect-[4/3]" />
            <ParallaxImg src={IMAGES.leaf} alt="Fresh green leaves" className="aspect-[4/3]" />
            <div className="absolute -left-10 top-1/2 hidden h-28 w-28 -translate-y-1/2 items-center justify-center rounded-full bg-white text-brand-dark shadow-[0_20px_40px_-20px_rgba(22,53,42,0.4)] md:flex">
              <Badge text="Offline first · Made for Bangladesh · " className="absolute inset-1.5" />
              <span className="text-lg font-semibold">M.</span>
            </div>
          </div>
        </div>
      </section>

      {/* Facts */}
      <section className="border-y border-line">
        <div className={`${WRAP} grid grid-cols-2 md:grid-cols-4 md:divide-x md:divide-line`}>
          {FACTS.map(([value, label], index) => (
            <Reveal key={label} className="py-10 pr-4 md:px-8 md:py-12 md:first:pl-0">
              <Counter value={value} className="text-5xl font-semibold tracking-tighter text-brand-dark md:text-6xl" />
              <p className="mt-3 max-w-[22ch] text-sm text-muted">{label}</p>
            </Reveal>
          ))}
        </div>
      </section>

      {/* Areas */}
      <section className={`${WRAP} py-24`}>
        <div className="grid gap-6 md:grid-cols-12 md:items-end">
          <Words text="Built for every part of the counter." accent="the counter." className="max-w-[18ch] text-4xl font-semibold leading-[1.02] tracking-tighter text-brand-dark md:col-span-7 md:text-6xl" />
          <Reveal className="md:col-span-4 md:col-start-9">
            <p className="text-muted">From the first sale of the morning to the expiry check at night, every job sits in one calm window.</p>
          </Reveal>
        </div>
        <Reveal className="mt-14 grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
          {AREAS.map((area) => (
            <Link key={area.label} to="/features" go={go} className="group relative block aspect-[4/5] overflow-hidden rounded-[1.75rem] bg-cream">
              <img src={area.image} alt={area.alt} loading="lazy" className="absolute inset-0 h-full w-full object-cover transition duration-[1400ms] ease-[cubic-bezier(0.16,1,0.3,1)] group-hover:scale-[1.06]" />
              <div className="absolute inset-0 bg-gradient-to-t from-brand-dark/75 via-brand-dark/5 to-transparent" />
              <div className="absolute inset-x-0 bottom-0 flex items-end justify-between p-5 text-white">
                <div>
                  <p className="text-xs uppercase tracking-[0.14em] text-white/70">{area.label}</p>
                  <p className="mt-1 text-xl font-medium tracking-tight">{area.text}</p>
                </div>
                <ArrowButton />
              </div>
            </Link>
          ))}
        </Reveal>
      </section>

      {/* Statement */}
      <section className={`${WRAP} grid gap-10 md:grid-cols-12 md:items-center`}>
        <ParallaxImg src={IMAGES.hands} alt="Two hands resting together on a counter" className="aspect-[4/3] w-full max-w-[540px] md:col-span-6" strength={4} />
        <div className="md:col-span-5 md:col-start-8">
          <Eyebrow>Why M. Pharmacy</Eyebrow>
          <Words text="Calm software for a busy counter." accent="busy counter." className="mt-6 text-4xl font-semibold leading-[1.02] tracking-tighter text-brand-dark md:text-[56px]" />
          <Reveal>
            <p className="mt-6 max-w-[46ch] leading-relaxed text-muted">
              No server in the shop, no technician and no setup fee. Every sale lives on your own computer first, so the counter keeps running through load-shedding and bad connections.
            </p>
            <Link to="/features" go={go} className="group mt-8 inline-flex items-center gap-3 rounded-full border border-line py-2 pl-5 pr-2 text-sm font-medium text-brand-dark transition hover:border-brand-dark">
              See every feature <ArrowButton dark />
            </Link>
          </Reveal>
        </div>
      </section>

      {/* Product */}
      <section className="mt-28 bg-cream py-24">
        <div className={WRAP}>
          <div className="grid gap-6 md:grid-cols-12 md:items-end">
            <Words text="Everything the counter needs, in one window." accent="one window." className="max-w-[20ch] text-4xl font-semibold leading-[1.02] tracking-tighter text-brand-dark md:col-span-8 md:text-6xl" />
            <Reveal className="md:col-span-3 md:col-start-10">
              <p className="text-muted">The desktop app, as it looks on your shop computer.</p>
            </Reveal>
          </div>
          <ScaleIn className="mx-auto mt-16 max-w-[1080px]">
            <Laptop />
          </ScaleIn>
          <Reveal className="mt-16 grid gap-8 sm:grid-cols-2 lg:grid-cols-4">
            {[
              [WifiSlash, "Works offline", "Sell for up to 30 days without internet."],
              [Package, "Batch and expiry", "Sells the oldest batch first and warns before a loss."],
              [Barcode, "Fast purchases", "Receive supplier invoices batch by batch."],
              [CloudCheck, "Owner dashboard", "Today's sales on your phone, anywhere."],
            ].map(([Icon, title, text]) => (
              <div key={title} className="border-t border-brand-dark/15 pt-5">
                <Icon size={26} weight="light" className="text-brand" />
                <p className="mt-4 font-medium text-brand-dark">{title}</p>
                <p className="mt-1 text-sm leading-relaxed text-muted">{text}</p>
              </div>
            ))}
          </Reveal>
        </div>
      </section>

      {/* Steps */}
      <section className={`${WRAP} grid gap-12 pt-28 md:grid-cols-12`}>
        <div className="md:col-span-4">
          <Eyebrow>How it works</Eyebrow>
          <Words text="Selling by this afternoon." accent="this afternoon." className="mt-6 text-4xl font-semibold leading-[1.02] tracking-tighter text-brand-dark md:text-5xl" />
          <Reveal>
            <ParallaxImg src={IMAGES.sprout} alt="A young plant growing on stone" className="mt-10 hidden aspect-[4/3] max-w-[420px] md:block" strength={5} />
          </Reveal>
        </div>
        <Reveal className="divide-y divide-line border-y border-line md:col-span-7 md:col-start-6">
          {STEPS.map(([title, text], index) => (
            <div key={title} className="grid grid-cols-[56px_1fr] items-baseline gap-4 py-8 md:grid-cols-[80px_200px_1fr]">
              <span className="text-sm tabular-nums text-brand">0{index + 1}</span>
              <p className="text-2xl font-semibold tracking-tight text-brand-dark md:text-3xl">{title}</p>
              <p className="col-start-2 text-muted md:col-start-3">{text}</p>
            </div>
          ))}
        </Reveal>
      </section>
    </main>
  );
}
