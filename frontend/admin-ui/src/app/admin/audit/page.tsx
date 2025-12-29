"use client";

import { useEffect, useMemo, useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { listAuditLogs } from "@/lib/audit";
import { getActiveAccessToken } from "@/lib/auth";
import type { AuditLog } from "@/gen/ecommerce/v1/audit_pb";

function formatTimestamp(ts?: { seconds?: string | number | bigint; nanos?: number }) {
  if (!ts || ts.seconds == null) {
    return "-";
  }
  const seconds = typeof ts.seconds === "bigint" ? Number(ts.seconds) : Number(ts.seconds);
  if (!Number.isFinite(seconds)) {
    return "-";
  }
  const date = new Date(seconds * 1000);
  return date.toLocaleString("ja-JP");
}

export default function AuditLogsPage() {
  const [logs, setLogs] = useState<AuditLog[]>([]);
  const [action, setAction] = useState("");
  const [actorId, setActorId] = useState("");
  const [actorType, setActorType] = useState("");
  const [targetType, setTargetType] = useState("");
  const [targetId, setTargetId] = useState("");
  const [pageToken, setPageToken] = useState("");
  const [nextPageToken, setNextPageToken] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const { push } = useToast();

  const hasFilter = useMemo(
    () =>
      action.trim() ||
      actorId.trim() ||
      actorType.trim() ||
      targetType.trim() ||
      targetId.trim(),
    [action, actorId, actorType, targetType, targetId]
  );

  async function loadLogs(options?: { resetPage?: boolean }) {
    if (!getActiveAccessToken()) {
      push({
        variant: "error",
        title: "Load failed",
        description: "access_token is missing. Please sign in first.",
      });
      return;
    }
    const nextToken = options?.resetPage ? "" : pageToken;
    setIsLoading(true);
    try {
      const data = await listAuditLogs({
        action: action.trim() || undefined,
        actorId: actorId.trim() || undefined,
        actorType: actorType.trim() || undefined,
        targetType: targetType.trim() || undefined,
        targetId: targetId.trim() || undefined,
        pageToken: nextToken,
      });
      setLogs(data.logs ?? []);
      setNextPageToken(data.page?.nextPageToken ?? "");
      setPageToken(nextToken);
    } catch (err) {
      push({
        variant: "error",
        title: "Load failed",
        description: err instanceof Error ? err.message : "Failed to load audit logs",
      });
    } finally {
      setIsLoading(false);
    }
  }

  useEffect(() => {
    void loadLogs({ resetPage: true });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <div className="space-y-6">
      <div>
        <div className="text-xs uppercase tracking-[0.3em] text-neutral-400">Admin</div>
        <h1 className="mt-2 text-2xl font-semibold text-neutral-900">Audit Logs</h1>
        <p className="mt-2 text-sm text-neutral-600">
          Review configuration changes and system actions.
        </p>
      </div>

      <Card className="border-neutral-200 bg-white text-neutral-900">
        <CardHeader>
          <CardTitle>Filters</CardTitle>
          <CardDescription className="text-neutral-500">
            Narrow down audit logs by action, actor, or target.
          </CardDescription>
        </CardHeader>
        <CardContent className="grid gap-4 md:grid-cols-3">
          <div className="space-y-2">
            <Label htmlFor="filterAction">Action</Label>
            <Input id="filterAction" value={action} onChange={(e) => setAction(e.target.value)} />
          </div>
          <div className="space-y-2">
            <Label htmlFor="filterActorId">Actor ID</Label>
            <Input id="filterActorId" value={actorId} onChange={(e) => setActorId(e.target.value)} />
          </div>
          <div className="space-y-2">
            <Label htmlFor="filterActorType">Actor Type</Label>
            <Input id="filterActorType" value={actorType} onChange={(e) => setActorType(e.target.value)} />
          </div>
          <div className="space-y-2">
            <Label htmlFor="filterTargetType">Target Type</Label>
            <Input id="filterTargetType" value={targetType} onChange={(e) => setTargetType(e.target.value)} />
          </div>
          <div className="space-y-2">
            <Label htmlFor="filterTargetId">Target ID</Label>
            <Input id="filterTargetId" value={targetId} onChange={(e) => setTargetId(e.target.value)} />
          </div>
          <div className="flex items-end gap-2">
            <Button type="button" onClick={() => loadLogs({ resetPage: true })} disabled={isLoading}>
              {isLoading ? "Loading..." : "Apply Filters"}
            </Button>
            {hasFilter ? (
              <Button
                type="button"
                variant="outline"
                onClick={() => {
                  setAction("");
                  setActorId("");
                  setActorType("");
                  setTargetType("");
                  setTargetId("");
                  setPageToken("");
                  void loadLogs({ resetPage: true });
                }}
              >
                Clear
              </Button>
            ) : null}
          </div>
        </CardContent>
      </Card>

      <Card className="border-neutral-200 bg-white text-neutral-900">
        <CardHeader>
          <CardTitle>Recent Logs</CardTitle>
          <CardDescription className="text-neutral-500">
            Latest audit events for this tenant.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3 text-sm text-neutral-700">
          {logs.length === 0 ? (
            <div className="text-sm text-neutral-600">No audit logs found.</div>
          ) : (
            logs.map((log) => (
              <div key={log.id} className="rounded-lg border border-neutral-200 px-4 py-3">
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <div className="font-medium text-neutral-900">{log.action}</div>
                  <div className="text-xs text-neutral-500">{formatTimestamp(log.createdAt)}</div>
                </div>
                <div className="mt-1 text-xs text-neutral-500">
                  actor: {log.actorType || "-"} {log.actorId || "-"}
                </div>
                <div className="text-xs text-neutral-500">
                  target: {log.targetType || "-"} {log.targetId || "-"}
                </div>
                {log.requestId ? (
                  <div className="text-xs text-neutral-400">request: {log.requestId}</div>
                ) : null}
              </div>
            ))
          )}
        </CardContent>
      </Card>

      <div className="flex items-center justify-end gap-2">
        <Button
          type="button"
          variant="outline"
          onClick={() => loadLogs({ resetPage: true })}
          disabled={isLoading}
        >
          Refresh
        </Button>
        <Button
          type="button"
          onClick={() => {
            if (!nextPageToken) {
              return;
            }
            setPageToken(nextPageToken);
            void loadLogs();
          }}
          disabled={isLoading || !nextPageToken}
        >
          Next Page
        </Button>
      </div>
    </div>
  );
}
