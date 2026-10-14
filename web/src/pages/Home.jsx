import { CloudCheck, Package, Receipt, WifiSlash } from "@phosphor-icons/react";
import { Laptop, Phone } from "../Mock.jsx";
import { BRAND, Link, Reveal, SITE, dark } from "../ui.jsx";

const textLink = "inline-flex items-center gap-1 text-[17px] text-brand hover:underline underline-offset-4";

export default function Home({ go }) {
  return (
    <main>
      {/* Hero */}
      <section className="px-4 pt-20 text-center sm:px-8 md:pt-28">
        <Reveal>
          <p className="text-[17px] font-medium text-brand">{BRAND}</p>
          <h1 className="mx-auto mt-3 max-w-4xl text-5xl font-semibold leading-[1.02] tracking-tighter md:text-[80px]">
            The pharmacy counter. Simplified.
          </h1>
          <p className="mx-auto mt-6 max-w-[44ch] text-xl leading-relaxed text-muted">
            Sales, stock and expiry on your shop computer. Works without internet.
          </p>
          <div className="mt-8 flex flex-wrap items-center justify-center gap-6">
            <Link to="/signup" go={go} className={`${dark} px-6 py-3 text-[15px]`}>Start free for a month</Link>
            <a href={SITE.downloadUrl} className={textLink}>Download for Windows &gt;</a>
          </div>
        </Reveal>
        <Reveal className="mx-auto mt-16 max-w-[1100px] md:mt-20" delay={250}>
          <Laptop />
        </Reveal>
      </section>

      {/* Offline */}
      <section className="mt-32 px-4 text-center sm:px-8">
        <Reveal>
          <WifiSlash size={44} weight="light" className="mx-auto text-brand" />
          <h2 className="mx-auto mt-6 max-w-3xl text-4xl font-semibold leading-[1.05] tracking-tighter md:text-6xl">
            No internet? <span className="text-muted">No problem.</span>
          </h2>
          <p className="mx-auto mt-6 max-w-[52ch] text-lg leading-relaxed text-muted">
            Keep selling through load-shedding and bad connections for up to 30 days. Everything syncs to the cloud the moment you're back online.
          </p>
        </Reveal>
      </section>

      {/* Tiles */}
      <section className="mt-24 grid gap-4 px-4 sm:px-8 md:grid-cols-2 lg:px-12">
        <Reveal className="flex min-h-[460px] flex-col justify-between rounded-[2rem] bg-cream p-10 md:p-14">
          <div>
            <Package size={36} weight="light" className="text-brand" />
            <h3 className="mt-6 max-w-sm text-3xl font-semibold leading-tight tracking-tight md:text-[40px]">Every batch. Every expiry.</h3>
            <p className="mt-4 max-w-[40ch] text-lg leading-relaxed text-muted">It sells the oldest batch first and warns you before medicine becomes a loss.</p>
          </div>
          <div className="mt-10 grid grid-cols-3 gap-3 text-sm">
            {[["Expires in 7 days", "4"], ["In 30 days", "17"], ["In 90 days", "52"]].map(([label, value]) => (
              <div key={label} className="rounded-2xl bg-white p-4">
                <p className="font-mono text-3xl tracking-tight">{value}</p>
                <p className="mt-1 text-xs text-muted">{label}</p>
              </div>
            ))}
          </div>
        </Reveal>
        <Reveal className="flex min-h-[460px] flex-col justify-between overflow-hidden rounded-[2rem] bg-ink p-10 text-white md:p-14">
          <div>
            <CloudCheck size={36} weight="light" className="text-brand-light" />
            <h3 className="mt-6 max-w-sm text-3xl font-semibold leading-tight tracking-tight md:text-[40px]">Your shop, on your phone.</h3>
            <p className="mt-4 max-w-[40ch] text-lg leading-relaxed text-white/60">See today's sales, stock and every computer from anywhere.</p>
          </div>
          <Phone className="mx-auto -mb-28 mt-10 w-[250px] translate-y-6" />
        </Reveal>
        <Reveal className="flex min-h-[360px] flex-col justify-between rounded-[2rem] bg-brand p-10 text-white md:col-span-2 md:flex-row md:items-end md:p-14">
          <div>
            <Receipt size={36} weight="light" className="text-white/80" />
            <h3 className="mt-6 max-w-xl text-3xl font-semibold leading-tight tracking-tight md:text-[40px]">The Bangladesh medicine catalogue, built in.</h3>
            <p className="mt-4 max-w-[48ch] text-lg leading-relaxed text-white/75">Search by brand or generic name. Updated for you by MEDITRONO, so you never type a medicine name again.</p>
          </div>
          <Link to="/features" go={go} className="mt-8 inline-flex text-[17px] text-white hover:underline underline-offset-4 md:mt-0">See all features &gt;</Link>
        </Reveal>
      </section>

      {/* Pricing teaser */}
      <section className="mt-32 px-4 text-center sm:px-8">
        <Reveal>
          <h2 className="mx-auto max-w-3xl text-4xl font-semibold leading-[1.05] tracking-tighter md:text-6xl">First month free.</h2>
          <p className="mx-auto mt-6 max-w-[46ch] text-lg leading-relaxed text-muted">
            One simple monthly price per computer after that. No setup fee, no card to start.
          </p>
          <div className="mt-8 flex flex-wrap items-center justify-center gap-6">
            <Link to="/signup" go={go} className={`${dark} px-6 py-3 text-[15px]`}>Start free</Link>
            <Link to="/pricing" go={go} className={textLink}>See pricing &gt;</Link>
          </div>
        </Reveal>
      </section>
    </main>
  );
}
