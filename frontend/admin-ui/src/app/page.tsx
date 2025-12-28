export default function Home() {
  return (
    <div className="min-h-screen bg-gradient-to-b from-neutral-50 to-neutral-100">
      <div className="mx-auto max-w-4xl px-6 py-16">
        <div className="rounded-2xl border border-neutral-200 bg-white p-8 shadow-sm">
          <div className="mb-6">
            <p className="text-xs uppercase tracking-[0.3em] text-neutral-400">
              rs-ecommerce
            </p>
            <h1 className="mt-2 text-3xl font-semibold text-neutral-900">
              Backoffice Console
            </h1>
            <p className="mt-2 text-sm text-neutral-600">
              Admin/Staff and Vendor backoffice entry points.
            </p>
          </div>
          <div className="grid gap-4 sm:grid-cols-2">
            <a
              href="/init"
              className="rounded-xl border border-neutral-200 bg-neutral-50 px-5 py-4 text-sm font-medium text-neutral-900 transition hover:bg-neutral-100"
            >
              Start Init Wizard
              <p className="mt-1 text-xs text-neutral-500">
                One-time setup for a new store.
              </p>
            </a>
            <a
              href="/login"
              className="rounded-xl border border-neutral-200 bg-white px-5 py-4 text-sm font-medium text-neutral-900 transition hover:bg-neutral-50"
            >
              Admin/Staff Login
              <p className="mt-1 text-xs text-neutral-500">
                Manage products, orders, promotions, and settings.
              </p>
            </a>
            <a
              href="/vendor/login"
              className="rounded-xl border border-neutral-200 bg-white px-5 py-4 text-sm font-medium text-neutral-900 transition hover:bg-neutral-50"
            >
              Vendor Login
              <p className="mt-1 text-xs text-neutral-500">
                Manage your shop inside the mall.
              </p>
            </a>
            <a
              href="/admin"
              className="rounded-xl border border-neutral-200 bg-neutral-900 px-5 py-4 text-sm font-medium text-white transition hover:bg-neutral-800"
            >
              Go to Admin Shell
              <p className="mt-1 text-xs text-neutral-300">
                Layout preview (no auth yet).
              </p>
            </a>
          </div>
        </div>
      </div>
    </div>
  );
}
