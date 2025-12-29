import StoreLocationForm from "@/components/store-location-form";

export default function LocationsPage() {
  return (
    <div>
      <h1 className="text-2xl font-semibold">Locations</h1>
      <p className="mt-2 text-sm text-neutral-400">Manage warehouse locations.</p>

      <div className="mt-8 grid gap-6 md:grid-cols-2">
        <StoreLocationForm />
      </div>
    </div>
  );
}
