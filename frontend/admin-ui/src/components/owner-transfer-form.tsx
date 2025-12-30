"use client";

import { useEffect, useMemo, useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { useToast } from "@/components/ui/toast";
import { identityListStaff, identityTransferOwner } from "@/lib/identity";
import { formatConnectError } from "@/lib/handle-error";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

type StaffOption = {
  staffId: string;
  displayName: string;
  email: string;
  loginId: string;
  phone: string;
  roleKey: string;
};

export default function OwnerTransferForm() {
  const [staff, setStaff] = useState<StaffOption[]>([]);
  const [selected, setSelected] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const { push } = useToast();

  const owner = useMemo(() => staff.find((item) => item.roleKey === "owner"), [staff]);
  const candidates = useMemo(() => staff.filter((item) => item.roleKey !== "owner"), [staff]);

  useEffect(() => {
    let cancelled = false;
    setIsLoading(true);
    identityListStaff()
      .then((data) => {
        if (cancelled) return;
        const list = (data.staff ?? []).map((item) => ({
          staffId: item.staffId,
          displayName: item.displayName ?? "",
          email: item.email ?? "",
          loginId: item.loginId ?? "",
          phone: item.phone ?? "",
          roleKey: item.roleKey ?? "",
        }));
        setStaff(list);
        if (!selected && list.length > 0) {
          const firstCandidate = list.find((item) => item.roleKey !== "owner");
          setSelected(firstCandidate?.staffId ?? "");
        }
      })
      .catch((err) => {
        if (!cancelled) {
          const uiError = formatConnectError(err, "Load failed", "Failed to load staff");
          push({
            variant: "error",
            title: uiError.title,
            description: uiError.description,
          });
        }
      })
      .finally(() => {
        if (!cancelled) {
          setIsLoading(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  function formatLabel(item: StaffOption) {
    return item.displayName || item.email || item.loginId || item.phone || item.staffId;
  }

  async function handleTransfer() {
    if (!selected) {
      push({
        variant: "error",
        title: "Select staff",
        description: "Choose a staff member to transfer ownership.",
      });
      return;
    }
    const confirmed = window.confirm("Transfer owner role to selected staff?");
    if (!confirmed) return;

    setIsSubmitting(true);
    try {
      const resp = await identityTransferOwner({ newOwnerStaffId: selected });
      if (!resp.transferred) {
        throw new Error("Transfer failed");
      }
      push({
        variant: "success",
        title: "Owner transferred",
        description: "Owner role has been transferred.",
      });
      const refreshed = await identityListStaff();
      const list = (refreshed.staff ?? []).map((item) => ({
        staffId: item.staffId,
        displayName: item.displayName ?? "",
        email: item.email ?? "",
        loginId: item.loginId ?? "",
        phone: item.phone ?? "",
        roleKey: item.roleKey ?? "",
      }));
      setStaff(list);
      const nextCandidate = list.find((item) => item.roleKey !== "owner");
      setSelected(nextCandidate?.staffId ?? "");
    } catch (err) {
      const uiError = formatConnectError(err, "Transfer failed", "Failed to transfer owner");
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
    <Card className="border-neutral-200 bg-white text-neutral-900">
      <CardHeader>
        <CardTitle>Transfer Owner</CardTitle>
        <CardDescription className="text-neutral-500">
          Transfer owner role to another staff. Owner only.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="text-sm text-neutral-600">
          Current owner: <span className="font-medium text-neutral-900">{owner ? formatLabel(owner) : "-"}</span>
        </div>
        <div className="space-y-1">
          <Label className="text-xs text-neutral-500">New owner</Label>
          <Select value={selected} onValueChange={setSelected} disabled={isLoading || candidates.length === 0}>
            <SelectTrigger className="bg-white">
              <SelectValue placeholder={isLoading ? "Loading staff..." : "Select staff"} />
            </SelectTrigger>
            <SelectContent>
              {candidates.length === 0 ? (
                <SelectItem value="__none__" disabled>
                  No candidates available
                </SelectItem>
              ) : (
                candidates.map((item) => (
                  <SelectItem key={item.staffId} value={item.staffId}>
                    {formatLabel(item)}
                  </SelectItem>
                ))
              )}
            </SelectContent>
          </Select>
        </div>
        <Button type="button" onClick={handleTransfer} disabled={isSubmitting || !selected || candidates.length === 0}>
          {isSubmitting ? "Transferring..." : "Transfer Owner"}
        </Button>
      </CardContent>
    </Card>
  );
}
