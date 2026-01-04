"use client";

import { useMemo, useState } from "react";
import { useRouter, useSearchParams } from "next/navigation";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { identityAcceptInvite } from "@/lib/identity";
import { formatConnectError } from "@/lib/handle-error";

export default function AcceptInvitePage() {
  const router = useRouter();
  const params = useSearchParams();
  const token = useMemo(() => params.get("token") ?? "", [params]);
  const [displayName, setDisplayName] = useState("");
  const [password, setPassword] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const { push } = useToast();

  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setIsSubmitting(true);
    try {
      const resp = await identityAcceptInvite({
        token,
        password,
        displayName: displayName.trim(),
      });
      push({
        variant: "success",
        title: "Invite accepted",
        description: "Your account is now active. Please sign in.",
      });
      router.push("/login");
      return resp;
    } catch (err) {
      const uiError = formatConnectError(err, "Accept failed", "Failed to accept invite");
      push({
        variant: "error",
        title: uiError.title,
        description: uiError.description,
      });
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <div className="min-h-screen bg-gradient-to-b from-neutral-50 to-neutral-100">
      <div className="mx-auto flex max-w-lg flex-col gap-6 px-6 py-16">
        <div>
          <p className="text-xs uppercase tracking-[0.3em] text-neutral-400">rs-ecommerce</p>
          <h1 className="mt-2 text-3xl font-semibold text-neutral-900">Accept Staff Invite</h1>
          <p className="mt-2 text-sm text-neutral-600">
            Set your password to activate the invited staff account.
          </p>
        </div>

        <Card>
          <CardHeader>
            <CardTitle>Activate account</CardTitle>
            <CardDescription>Confirm invite token and set a password.</CardDescription>
          </CardHeader>
          <CardContent>
            <form className="space-y-4" onSubmit={handleSubmit}>
              <div className="space-y-2">
                <div className="rounded-md border border-dashed border-neutral-200 bg-neutral-50 px-3 py-2 text-xs text-neutral-500">
                  Invite token is detected from the URL.
                </div>
              </div>
              <div className="space-y-2">
                <Label htmlFor="displayName">Display name</Label>
                <Input
                  id="displayName"
                  value={displayName}
                  onChange={(e) => setDisplayName(e.target.value)}
                  placeholder="Your name"
                  required
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="password">Password</Label>
                <Input
                  id="password"
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder="Set a password"
                  required
                />
              </div>
              <Button type="submit" className="w-full" disabled={isSubmitting || !token}>
                {isSubmitting ? "Submitting..." : "Accept invite"}
              </Button>
            </form>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
