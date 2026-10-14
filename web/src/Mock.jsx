// Product shots drawn in HTML, mirroring the desktop app's real layout
// (green sidebar, white panel, dashboard tiles). Example figures only.

const MENU = ["Dashboard", "Sale", "Stock", "Purchase", "Catalogue", "History", "Settings"];
const WEEK = [62, 48, 71, 55, 83, 77, 94];
const LOW = [
  ["Napa 500 mg", "Tablet", "12 left"],
  ["Seclo 20 mg", "Capsule", "8 left"],
  ["Monas 10 mg", "Tablet", "15 left"],
];

function Chart({ height = 120 }) {
  const max = Math.max(...WEEK);
  const points = WEEK.map((v, i) => `${(i / (WEEK.length - 1)) * 100},${100 - (v / max) * 88}`).join(" ");
  return (
    <svg viewBox="0 0 100 100" preserveAspectRatio="none" className="w-full" style={{ height }} aria-hidden="true">
      <defs>
        <linearGradient id="area" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0" stopColor="#2f8a63" stopOpacity="0.22" />
          <stop offset="1" stopColor="#2f8a63" stopOpacity="0" />
        </linearGradient>
      </defs>
      <polygon points={`0,100 ${points} 100,100`} fill="url(#area)" />
      <polyline points={points} fill="none" stroke="#2f8a63" strokeWidth="1.6" vectorEffect="non-scaling-stroke" />
    </svg>
  );
}

export function DesktopMock() {
  return (
    <div className="flex aspect-[16/10] w-full overflow-hidden rounded-[10px] bg-brand text-[clamp(6px,0.75vw,11px)] text-ink">
      <aside className="flex w-[18%] flex-col gap-[0.4em] p-[1.4em] text-white">
        <p className="mb-[1.2em] text-[1.25em] font-semibold">M. Pharmacy</p>
        {MENU.map((item, index) => (
          <span key={item} className={`rounded-[0.6em] px-[0.9em] py-[0.6em] ${index === 0 ? "bg-white font-medium text-brand" : "text-white/80"}`}>{item}</span>
        ))}
      </aside>
      <main className="m-[0.8em] ml-0 flex-1 rounded-[1em] bg-white p-[1.8em]">
        <div className="flex items-baseline justify-between">
          <p className="text-[1.6em] font-semibold tracking-tight">Good morning</p>
          <span className="flex items-center gap-[0.4em] text-brand">
            <span className="h-[0.5em] w-[0.5em] rounded-full bg-brand" /> Offline, all saved
          </span>
        </div>
        <div className="mt-[1.4em] grid grid-cols-4 gap-[0.8em]">
          {[["Today's sales", "৳18,460"], ["Bills", "64"], ["Expiring in 30 days", "17"], ["Low stock", "9"]].map(([label, value]) => (
            <div key={label} className="rounded-[0.9em] bg-cream p-[1em]">
              <p className="text-muted">{label}</p>
              <p className="mt-[0.3em] text-[1.7em] font-semibold tracking-tight">{value}</p>
            </div>
          ))}
        </div>
        <div className="mt-[0.8em] grid grid-cols-[1.6fr_1fr] gap-[0.8em]">
          <div className="rounded-[0.9em] border border-line p-[1em]">
            <p className="font-medium">Sales this week</p>
            <Chart height="9em" />
          </div>
          <div className="rounded-[0.9em] border border-line p-[1em]">
            <p className="font-medium">Reorder soon</p>
            <div className="mt-[0.6em] divide-y divide-line">
              {LOW.map(([name, form, left]) => (
                <div key={name} className="flex justify-between py-[0.55em]">
                  <span>{name} <span className="text-muted">{form}</span></span>
                  <span className="text-brand">{left}</span>
                </div>
              ))}
            </div>
          </div>
        </div>
      </main>
    </div>
  );
}

export function Laptop({ className = "" }) {
  return (
    <div className={className}>
      <div className="rounded-[18px] border border-ink/10 bg-ink p-[1.2%] shadow-[0_40px_80px_-30px_rgba(12,33,24,0.45)]">
        <DesktopMock />
      </div>
      <div className="mx-auto h-3 w-[112%] -translate-x-[5.4%] rounded-b-[14px] bg-gradient-to-b from-[#c9d8cf] to-[#a9bdb1]" />
    </div>
  );
}

export function Phone({ className = "" }) {
  return (
    <div className={`rounded-[2.6rem] border border-ink/10 bg-ink p-2 shadow-[0_40px_80px_-30px_rgba(12,33,24,0.45)] ${className}`}>
      <div className="overflow-hidden rounded-[2.1rem] bg-white px-5 pb-6 pt-10 text-ink">
        <div className="mx-auto -mt-7 mb-6 h-5 w-24 rounded-full bg-ink" />
        <p className="text-xs text-muted">Rahman Pharmacy</p>
        <p className="mt-1 text-2xl font-semibold tracking-tight">৳18,460</p>
        <p className="text-xs text-brand">Today, up from ৳16,920 yesterday</p>
        <div className="mt-4 rounded-2xl bg-cream p-3">
          <Chart height={70} />
        </div>
        <div className="mt-3 grid grid-cols-2 gap-2 text-xs">
          <div className="rounded-2xl bg-cream p-3"><p className="text-muted">Expiring</p><p className="mt-1 text-lg font-semibold">17</p></div>
          <div className="rounded-2xl bg-cream p-3"><p className="text-muted">Low stock</p><p className="mt-1 text-lg font-semibold">9</p></div>
        </div>
        <div className="mt-3 rounded-2xl border border-line p-3 text-xs">
          <p className="font-medium">Computers</p>
          <div className="mt-2 flex justify-between"><span>Counter PC</span><span className="text-brand">Active</span></div>
          <div className="mt-1 flex justify-between"><span>Back office</span><span className="text-brand">Active</span></div>
        </div>
      </div>
    </div>
  );
}
