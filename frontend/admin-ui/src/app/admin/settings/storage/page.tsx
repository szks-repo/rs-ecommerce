import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import AdminPageHeader from "@/components/admin-page-header";

export default function SettingsStoragePage() {
  return (
    <div className="space-y-6">
      <AdminPageHeader
        title="Storage"
        description="Storage is configured by the system operator, not per store."
      />
      <Alert className="border-sky-200 bg-sky-50 text-sky-900">
        <AlertTitle>How storage works</AlertTitle>
        <AlertDescription>
          <div className="space-y-2 text-sm">
            <p>
              Public storage is used for storefront assets (product images, thumbnails). Private
              storage is used for restricted downloads (digital goods).
            </p>
            <p>
              Object keys are prefixed with <code>{`{tenant_id}/{store_id}`}</code> and an optional
              base path from the environment configuration.
            </p>
          </div>
        </AlertDescription>
      </Alert>
      <Alert>
        <AlertTitle>Operator configuration</AlertTitle>
        <AlertDescription>
          <div className="space-y-2 text-sm text-neutral-600">
            <p>Set these environment variables on the server:</p>
            <ul className="list-disc pl-5">
              <li>STORAGE_PUBLIC_PROVIDER / STORAGE_PUBLIC_BUCKET / STORAGE_PUBLIC_BASE_PATH</li>
              <li>STORAGE_PUBLIC_CDN_BASE_URL / STORAGE_PUBLIC_REGION</li>
              <li>STORAGE_PRIVATE_PROVIDER / STORAGE_PRIVATE_BUCKET / STORAGE_PRIVATE_BASE_PATH</li>
              <li>STORAGE_PRIVATE_CDN_BASE_URL / STORAGE_PRIVATE_REGION</li>
            </ul>
            <p>
              For GCS, provide <code>GOOGLE_APPLICATION_CREDENTIALS</code> on the server. For S3,
              configure AWS credentials and endpoint as usual.
            </p>
          </div>
        </AlertDescription>
      </Alert>
    </div>
  );
}
