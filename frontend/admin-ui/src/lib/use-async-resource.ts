"use client";

import { useCallback, useEffect, useState, type DependencyList } from "react";

type AsyncState<T> = {
  data: T | null;
  error: unknown;
  loading: boolean;
};

export function useAsyncResource<T>(
  loader: () => Promise<T>,
  deps: DependencyList
) {
  const [state, setState] = useState<AsyncState<T>>({
    data: null,
    error: null,
    loading: true,
  });

  const load = useCallback(async () => {
    setState((prev) => ({ ...prev, loading: true, error: null }));
    try {
      const data = await loader();
      setState({ data, error: null, loading: false });
    } catch (error) {
      setState({ data: null, error, loading: false });
    }
  }, deps);

  useEffect(() => {
    void load();
  }, [load]);

  return { ...state, reload: load };
}
