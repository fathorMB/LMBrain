import { useCallback, useEffect, useRef, useState, type DependencyList } from "react";

export interface AsyncResourceOptions<T> {
  initialData?: T | null;
  enabled?: boolean;
  onSuccess?: (data: T) => void;
  onError?: (error: Error) => void;
}

export interface AsyncResourceResult<T> {
  data: T | null;
  loading: boolean;
  error: string | null;
  refresh: () => Promise<T | null>;
  setData: (data: T | null | ((prev: T | null) => T | null)) => void;
}

function errorMessage(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  return "An unexpected error occurred.";
}

export function useAsyncResource<T>(
  fetcher: () => Promise<T>,
  deps: DependencyList = [],
  options: AsyncResourceOptions<T> = {}
): AsyncResourceResult<T> {
  const { initialData = null, enabled = true, onSuccess, onError } = options;

  const [data, setData] = useState<T | null>(initialData);
  const [loading, setLoading] = useState<boolean>(enabled);
  const [error, setError] = useState<string | null>(null);

  const fetcherRef = useRef(fetcher);
  fetcherRef.current = fetcher;

  const onSuccessRef = useRef(onSuccess);
  onSuccessRef.current = onSuccess;

  const onErrorRef = useRef(onError);
  onErrorRef.current = onError;

  const refresh = useCallback(async (): Promise<T | null> => {
    setLoading(true);
    setError(null);
    try {
      const result = await fetcherRef.current();
      setData(result);
      onSuccessRef.current?.(result);
      return result;
    } catch (err) {
      const msg = errorMessage(err);
      setError(msg);
      onErrorRef.current?.(err instanceof Error ? err : new Error(msg));
      return null;
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    if (!enabled) {
      setLoading(false);
      return;
    }

    let active = true;
    setLoading(true);
    setError(null);

    fetcherRef
      .current()
      .then((result) => {
        if (!active) return;
        setData(result);
        onSuccessRef.current?.(result);
      })
      .catch((err) => {
        if (!active) return;
        const msg = errorMessage(err);
        setError(msg);
        onErrorRef.current?.(err instanceof Error ? err : new Error(msg));
      })
      .finally(() => {
        if (active) setLoading(false);
      });

    return () => {
      active = false;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [enabled, ...deps]);

  return {
    data,
    loading,
    error,
    refresh,
    setData,
  };
}
