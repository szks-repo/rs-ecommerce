"use client";

import { useCallback } from "react";
import { useToast } from "@/components/ui/toast";
import { formatConnectError } from "@/lib/handle-error";

type ToastMessage = {
  title?: string;
  description: string;
  variant: "success" | "error";
};

type ApiCallOptions = {
  success?: Omit<ToastMessage, "variant">;
  errorTitle?: string;
  errorDescription?: string;
  throwOnError?: boolean;
};

export function useApiCall() {
  const { push } = useToast();

  const call = useCallback(
    async <T>(fn: () => Promise<T>, options?: ApiCallOptions): Promise<T | null> => {
      try {
        const result = await fn();
        if (options?.success) {
          push({
            title: options.success.title,
            description: options.success.description,
            variant: "success",
          });
        }
        return result;
      } catch (err) {
        const uiError = formatConnectError(
          err,
          options?.errorTitle || "Request failed",
          options?.errorDescription || "Unknown error"
        );
        push(uiError);
        if (options?.throwOnError) {
          throw err;
        }
        return null;
      }
    },
    [push]
  );

  const notifyError = useCallback(
    (err: unknown, title: string, description: string) => {
      const uiError = formatConnectError(err, title, description);
      push(uiError);
    },
    [push]
  );

  return { call, notifyError };
}
