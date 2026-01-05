"use client";

import Link from "next/link";
import { useDeferredValue, useEffect, useMemo, useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { Input } from "@/components/ui/input";
import DateCell from "@/components/date-cell";
import { identityListRoles, identityListStaff } from "@/lib/identity";
import { useApiCall } from "@/lib/use-api-call";
import { useAsyncResource } from "@/lib/use-async-resource";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

type StaffRow = {
  staffId: string;
  email: string;
  loginId: string;
  phone: string;
  roleId: string;
  roleKey: string;
  status: string;
  displayName: string;
  createdAt?: { seconds?: string | number | bigint; nanos?: number };
};

type RoleRow = {
  id: string;
  key: string;
  name: string;
};

export default function StaffListForm() {
  const [staff, setStaff] = useState<StaffRow[]>([]);
  const [roles, setRoles] = useState<RoleRow[]>([]);
  const [query, setQuery] = useState("");
  const deferredQuery = useDeferredValue(query);
  const [roleFilter, setRoleFilter] = useState("all");
  const [statusFilter, setStatusFilter] = useState("all");
  const [pageSize, setPageSize] = useState(50);
  const [page, setPage] = useState(1);
  const { push } = useToast();
  const { notifyError } = useApiCall();
  const { data, loading, error, reload } = useAsyncResource<{
    roles: RoleRow[];
    staff: StaffRow[];
    page: number;
    pageSize: number;
    total: number;
  }>(async () => {
    const [roleResp, staffResp] = await Promise.all([
      identityListRoles(),
      identityListStaff({
        page,
        pageSize,
        query: deferredQuery.trim(),
        roleId: roleFilter === "all" ? "" : roleFilter,
        status: statusFilter === "all" ? "" : statusFilter,
      }),
    ]);
    const roleRows = roleResp.roles ?? [];
    const list = (staffResp.staff ?? []).map((item) => ({
      staffId: item.staffId,
      email: item.email ?? "",
      loginId: item.loginId ?? "",
      phone: item.phone ?? "",
      roleId: item.roleId ?? "",
      roleKey: item.roleKey ?? "",
      status: item.status ?? "",
      displayName: item.displayName ?? "",
      createdAt: item.createdAt,
    }));
    list.sort((a, b) => {
      const aOwner = a.roleKey === "owner" ? 1 : 0;
      const bOwner = b.roleKey === "owner" ? 1 : 0;
      if (aOwner !== bOwner) {
        return bOwner - aOwner;
      }
      return a.staffId.localeCompare(b.staffId);
    });
    return {
      roles: roleRows,
      staff: list,
      page: staffResp.page ?? page,
      pageSize: staffResp.pageSize ?? pageSize,
      total: staffResp.total ?? list.length,
    };
  }, [page, pageSize, deferredQuery, roleFilter, statusFilter]);

  const roleLabelMap = useMemo(() => {
    const map = new Map<string, string>();
    roles.forEach((role) => {
      map.set(role.key, role.name || role.key);
    });
    return map;
  }, [roles]);

  const roleOptions = useMemo(() => {
    return roles.map((role) => ({
      id: role.id,
      label: role.name || role.key || role.id,
    }));
  }, [roles]);

  const statusOptions = useMemo(() => {
    const set = new Set<string>();
    staff.forEach((row) => {
      if (row.status) {
        set.add(row.status);
      }
    });
    return Array.from(set).sort();
  }, [staff]);

  useEffect(() => {
    if (error) {
      notifyError(error, "Load failed", "Failed to load staff");
      return;
    }
    if (!data) {
      return;
    }
    setRoles(data.roles);
    setStaff(data.staff);
  }, [data, error, notifyError]);

  function formatStaffLabel(row: StaffRow) {
    const primary = row.displayName || row.email || row.loginId || row.phone || row.staffId;
    return primary;
  }

  function formatStaffId(staffId: string) {
    if (staffId.length <= 12) {
      return staffId;
    }
    return `${staffId.slice(0, 8)}…${staffId.slice(-4)}`;
  }

  function toIsoString(ts?: { seconds?: string | number | bigint; nanos?: number }) {
    if (!ts?.seconds) {
      return "";
    }
    const seconds = typeof ts.seconds === "bigint" ? Number(ts.seconds) : Number(ts.seconds);
    if (!Number.isFinite(seconds)) {
      return "";
    }
    const date = new Date(seconds * 1000);
    return Number.isNaN(date.getTime()) ? "" : date.toISOString();
  }

  const total = data?.total ?? staff.length;
  const totalPages = Math.max(1, Math.ceil(total / pageSize));
  const currentPage = Math.min(page, totalPages);

  return (
    <Card className="border-neutral-200 bg-white text-neutral-900">
      <CardHeader>
        <CardTitle>Staff List</CardTitle>
        <CardDescription className="text-neutral-500">
          Search staff and open details to edit roles or profile.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex flex-wrap items-center justify-between gap-2 text-sm text-neutral-500">
          <div>
            {total} staff members
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <Input
              value={query}
              onChange={(e) => {
                setQuery(e.target.value);
                setPage(1);
              }}
              placeholder="Search by name, email, login_id, phone, staff_id"
              className="h-9 w-full min-w-[220px] max-w-[360px] bg-white"
            />
            <Select
              value={roleFilter}
              onValueChange={(value) => {
                setRoleFilter(value);
                setPage(1);
              }}
            >
              <SelectTrigger className="h-9 w-[180px] bg-white">
                <SelectValue placeholder="Role" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All roles</SelectItem>
                {roleOptions.map((role) => (
                  <SelectItem key={role.id} value={role.id}>
                    {role.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Select
              value={statusFilter}
              onValueChange={(value) => {
                setStatusFilter(value);
                setPage(1);
              }}
            >
              <SelectTrigger className="h-9 w-[160px] bg-white">
                <SelectValue placeholder="Status" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All status</SelectItem>
                {statusOptions.map((status) => (
                  <SelectItem key={status} value={status}>
                    {status}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Select
              value={String(pageSize)}
              onValueChange={(value) => {
                setPageSize(Number(value));
                setPage(1);
              }}
            >
              <SelectTrigger className="h-9 w-[120px] bg-white">
                <SelectValue placeholder="Rows" />
              </SelectTrigger>
              <SelectContent>
                {[25, 50, 100].map((size) => (
                  <SelectItem key={size} value={String(size)}>
                    {size} / page
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Button type="button" variant="outline" size="sm" onClick={reload} disabled={loading}>
              {loading ? "Loading..." : "Refresh"}
            </Button>
          </div>
        </div>
        <div className="space-y-3">
          {staff.length === 0 ? (
            <div className="text-sm text-neutral-600">No staff found.</div>
          ) : (
            <div className="overflow-hidden rounded-lg border border-neutral-200 bg-white">
              <div className="max-h-[520px] overflow-auto">
                <table className="min-w-full text-sm">
                  <thead className="sticky top-0 bg-neutral-50 text-xs uppercase text-neutral-500">
                    <tr>
                      <th className="px-3 py-2 text-left font-medium">Staff</th>
                      <th className="px-3 py-2 text-left font-medium">Contact</th>
                      <th className="px-3 py-2 text-left font-medium">Role</th>
                      <th className="px-3 py-2 text-left font-medium">Status</th>
                      <th className="px-3 py-2 text-left font-medium">Created</th>
                      <th className="px-3 py-2 text-right font-medium">Detail</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-neutral-200">
                    {staff.map((row) => {
                      const isOwner = row.roleKey === "owner";
                      return (
                        <tr key={row.staffId} className="align-top">
                          <td className="px-3 py-2">
                            <div className="flex flex-wrap items-center gap-2">
                              <div className="text-sm font-medium text-neutral-900">
                                {formatStaffLabel(row)}
                              </div>
                              {isOwner ? (
                                <span className="rounded-full bg-emerald-50 px-2 py-0.5 text-[10px] font-medium text-emerald-700">
                                  Owner
                                </span>
                              ) : null}
                            </div>
                            <div className="text-[11px] text-neutral-500">
                              id: {formatStaffId(row.staffId)}
                            </div>
                          </td>
                          <td className="px-3 py-2 text-[11px] text-neutral-600">
                            {row.email ? <div>{row.email}</div> : null}
                            {row.loginId ? <div>{row.loginId}</div> : null}
                            {!row.loginId && row.phone ? <div>{row.phone}</div> : null}
                          </td>
                          <td className="px-3 py-2 text-[11px] text-neutral-500">
                            {roleLabelMap.get(row.roleKey) ?? row.roleKey}
                          </td>
                          <td className="px-3 py-2 text-[11px] text-neutral-500">{row.status}</td>
                          <td className="px-3 py-2 text-[11px] text-neutral-500">
                            <DateCell value={toIsoString(row.createdAt)} />
                          </td>
                          <td className="px-3 py-2 text-right">
                            <Button asChild type="button" size="sm" variant="outline">
                              <Link href={`/admin/identity/${row.staffId}`}>Open</Link>
                            </Button>
                          </td>
                        </tr>
                      );
                    })}
                  </tbody>
                </table>
              </div>
            </div>
          )}
        </div>
        {total > pageSize ? (
          <div className="flex flex-wrap items-center justify-between gap-2 border-t border-neutral-200 pt-4 text-sm">
            <div className="text-neutral-500">
              Page {currentPage} / {totalPages}
            </div>
            <div className="flex items-center gap-2">
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={() => setPage((prev) => Math.max(1, prev - 1))}
                disabled={currentPage <= 1}
              >
                Prev
              </Button>
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={() => setPage((prev) => Math.min(totalPages, prev + 1))}
                disabled={currentPage >= totalPages}
              >
                Next
              </Button>
            </div>
          </div>
        ) : null}
      </CardContent>
    </Card>
  );
}
