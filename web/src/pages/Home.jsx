import { Barcode, CloudCheck, Package, ShieldCheck, WifiSlash } from "@phosphor-icons/react";
import { Laptop } from "../Mock.jsx";
import { Counter, Eyebrow, IMAGES, Link, Reveal, SITE, dark, light } from "../ui.jsx";

// Brand tiles are about 400px wide, so every image here is shown at most
// about 1.5x its natural size to stay sharp.
function Img({ src, alt = "", className = "" }) {
  return (
    <div className={`overflow-hidden rounded-[1.75rem] bg-cream ${className}`}>
      <img src={src} alt={alt} loading="lazy" className="h-full w-full object-cover transition duration-[1400ms] ease-[cubic-bezier(0.16,1,0.3,1)] hover:scale-[1.04]" />
    </div>
  );
}

const textLink = "text-[17px] text-brand underline-offset-4 hover:underline";

export default function Home({ go }) {
  return (
    <main>
      {/* Hero: words left, brand mosaic right */}
      <section className="grid items-center gap-12 px-4 pb-20 pt-14 sm:px-8 lg:grid-cols-[1fr_minmax(0,640px)] lg:px-12 lg:pt-20">
        <Reveal>
          <Eyebrow>Pharmacy software for Bangladesh</Eyebrow>
          <h1 className="mt-7 max-w-[13ch] text-5xl font-semibold leading-[1] tracking-tighter text-brand-dark md:text-7xl">
            Care at the counter, made simple.
          </h1>
          <p className="mt-6 max-w-[46ch] text-lg leading-relaxed text-muted">
            M. Pharmacy runs sales, stock and expiry on your shop computer. It keeps working without internet, and keeps a safe copy in the cloud.
          </p>
          <div className="mt-9 flex flex-wrap items-center gap-6">
            <Link to="/signup" go={go} className={`${dark} bg-brand-dark px-6 py-3 text-[15px]`}>Start your free month</Link>
            <a href={SITE.downloadUrl} className={textLink}>Download for Windows</a>
          </div>
        </Reveal>
        <Reveal className="grid grid-cols-2 gap-3" delay={150}>
          <Img src={IMAGES.shelves} alt="A bright pharmacy with stocked shelves" className="row-span-2 aspect-[3/4.2]" />
          <Img src={IMAGES.capsules} alt="Green and white capsules on a counter" className="aspect-[4/3]" />
          <Img src={IMAGES.scanner} alt="Scanning a medicine box at the counter" className="aspect-[4/3]" />
        </Reveal>
      </section>

      {/* Facts */}
      <Reveal as="section" className="grid grid-cols-2 border-y border-line md:grid-cols-4">
        {[["30", "days offline, no connection needed"], ["7", "day grace, never locked out"], ["৳0", "setup fee, first month free"], ["1", "Windows computer is enough"]].map(([value, label], index) => (
          <div key={label} className={`px-4 py-12 sm:px-8 lg:px-12 ${index ? "md:border-l md:border-line" : ""}`}>
            <Counter value={value} className="text-5xl tracking-tighter text-brand-dark md:text-6xl" />
            <p className="mt-3 max-w-[22ch] text-sm text-muted">{label}</p>
          </div>
        ))}
      </Reveal>

      {/* Statement */}
      <section className="grid items-center gap-12 px-4 py-24 sm:px-8 md:grid-cols-[minmax(0,560px)_1fr] lg:px-12">
        <Img src={IMAGES.leaf} alt="Fresh green leaves with morning dew" className="aspect-[3/2]" />
        <Reveal>
          <Eyebrow>Why M. Pharmacy</Eyebrow>
          <h2 className="mt-6 max-w-[16ch] text-4xl font-semibold leading-[1.02] tracking-tighter text-brand-dark md:text-6xl">Calm, natural and reliable.</h2>
          <p className="mt-6 max-w-[52ch] text-lg leading-relaxed text-muted">
            No server in the shop, no technician and no setup fee. The counter keeps running through load-shedding and bad connections.
          </p>
        </Reveal>
      </section>

      {/* Product */}
      <section className="bg-cream px-4 py-24 sm:px-8 lg:px-12">
        <Reveal className="mx-auto max-w-3xl text-center">
          <Eyebrow>The app</Eyebrow>
          <h2 className="mt-6 text-4xl font-semibold leading-[1.02] tracking-tighter text-brand-dark md:text-6xl">Everything the counter needs. In one window.</h2>
        </Reveal>
        <Reveal className="mx-auto mt-14 max-w-[1040px]" delay={150}>
          <Laptop />
        </Reveal>
        <Reveal className="mx-auto mt-16 grid max-w-[1040px] gap-8 sm:grid-cols-2 lg:grid-cols-4">
          {[
            [WifiSlash, "Works offline", "Sell for up to 30 days without internet."],
            [Package, "Batch and expiry", "Sells the oldest batch first, warns before loss."],
            [Barcode, "Fast purchases", "Receive supplier invoices batch by batch."],
            [CloudCheck, "Owner dashboard", "Today's sales on your phone, anywhere."],
          ].map(([Icon, title, text]) => (
            <div key={title} className="border-t border-line pt-5">
              <Icon size={26} weight="light" className="text-brand" />
              <p className="mt-4 font-medium text-brand-dark">{title}</p>
              <p className="mt-1 text-sm leading-relaxed text-muted">{text}</p>
            </div>
          ))}
        </Reveal>
      </section>

      {/* Trust and MEDITRONO */}
      <section className="grid gap-3 px-3 pt-3 md:grid-cols-2">
        <div className="grid grid-cols-[minmax(0,1fr)_minmax(0,1fr)] items-stretch gap-3 rounded-[2rem] bg-brand-soft p-3">
          <Img src={IMAGES.hands} alt="Two hands resting together on a counter" className="aspect-square" />
          <Reveal className="flex flex-col justify-center p-4 sm:p-6">
            <ShieldCheck size={30} weight="light" className="text-brand" />
            <h3 className="mt-5 text-2xl font-semibold leading-tight tracking-tight text-brand-dark md:text-3xl">Your data stays yours.</h3>
            <p className="mt-3 text-sm leading-relaxed text-muted">Every sale lives on your own computer first. MEDITRONO only sees it if you choose to share.</p>
          </Reveal>
        </div>
        <div className="grid grid-cols-[minmax(0,1fr)_minmax(0,1fr)] items-stretch gap-3 rounded-[2rem] bg-brand-dark p-3 text-white">
          <Reveal className="flex flex-col justify-center p-4 sm:p-6">
            <p className="text-xs uppercase tracking-[0.14em] text-white/60">Part of MEDITRONO</p>
            <h3 className="mt-5 text-2xl font-semibold leading-tight tracking-tight md:text-3xl">Bangladesh's digital health infrastructure.</h3>
            <Link to="/about" go={go} className="mt-5 text-sm text-brand-light underline-offset-4 hover:underline">About MEDITRONO</Link>
          </Reveal>
          <Img src={IMAGES.building} alt="A modern healthcare building among trees" className="aspect-square" />
        </div>
      </section>

      {/* Call to action */}
      <section className="px-3 pt-3">
        <div className="grid items-center gap-10 rounded-[2rem] bg-brand-dark px-6 py-12 text-white sm:px-12 md:grid-cols-[1fr_minmax(0,420px)]">
          <Reveal>
            <h2 className="max-w-[14ch] text-4xl font-semibold leading-[1.02] tracking-tighter md:text-6xl">Your first month is on us.</h2>
            <p className="mt-5 max-w-[44ch] text-white/70">No card, no setup fee. Join the free pilot in Naogaon.</p>
            <div className="mt-8 flex flex-wrap gap-3">
              <Link to="/signup" go={go} className={`${light} border-white py-3`}>Start free</Link>
              <Link to="/pricing" go={go} className="inline-flex items-center rounded-full border border-white/30 px-5 py-3 text-sm transition hover:bg-white/10 active:scale-[0.98]">See pricing</Link>
            </div>
          </Reveal>
          <Img src={IMAGES.sign} alt="The M. Pharmacy sign" className="aspect-square" />
        </div>
      </section>
    </main>
  );
}
