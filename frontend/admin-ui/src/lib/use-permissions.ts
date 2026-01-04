"use client";

import { useCallback, useEffect, useMemo, useState } from "react";
import { identityListMyPermissions } from "@/lib/identity";
import { getActiveActorInfo } from "@/lib/auth";
import {
  type PermissionKey,
  type PermissionKeyLiteral,
  normalizePermissionKeys,
} from "@/lib/permissions";

type PermissionsState = {
  status: "loading" | "ready" | "error";
  permissionKeys: PermissionKey[];
  roleKey?: string;
};

export function usePermissions() {
  const [state, setState] = useState<PermissionsState>({
    status: "loading",
    permissionKeys: [],
  });

  useEffect(() => {
    const actor = getActiveActorInfo();
    if (!actor) {
      setState({ status: "ready", permissionKeys: [] });
      return;
    }
    if (actor.role === "owner") {
      setState({
        status: "ready",
        permissionKeys: ["*" as PermissionKey],
        roleKey: "owner",
      });
      return;
    }
    identityListMyPermissions()
      .then((resp) => {
        const permissionKeys = normalizePermissionKeys(resp.permissionKeys || []);
        setState({
          status: "ready",
          permissionKeys,
          roleKey: resp.roleKey || undefined,
        });
      })
      .catch(() => {
        setState({ status: "error", permissionKeys: [] });
      });
  }, []);

  const has = useCallback(
    (permission?: PermissionKeyLiteral) => {
      if (!permission) {
        return true;
      }
      if (state.permissionKeys.includes("*" as PermissionKey)) {
        return true;
      }
      return state.permissionKeys.includes(permission as PermissionKey);
    },
    [state.permissionKeys]
  );

  const value = useMemo(() => ({ ...state, has }), [state, has]);
  return value;
}
