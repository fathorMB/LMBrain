import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { useAsyncResource } from "../hooks/useAsyncResource";

describe("useAsyncResource hook", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("fetches data and updates loading and data states on success", async () => {
    const fetcher = vi.fn().mockResolvedValue({ id: 1, name: "Test" });
    const onSuccess = vi.fn();

    const { result } = renderHook(() =>
      useAsyncResource(fetcher, [], { onSuccess })
    );

    expect(result.current.loading).toBe(true);
    expect(result.current.data).toBeNull();
    expect(result.current.error).toBeNull();

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.data).toEqual({ id: 1, name: "Test" });
    expect(result.current.error).toBeNull();
    expect(onSuccess).toHaveBeenCalledWith({ id: 1, name: "Test" });
  });

  it("handles fetch failure gracefully and updates error state", async () => {
    const fetcher = vi.fn().mockRejectedValue(new Error("Network failure"));
    const onError = vi.fn();

    const { result } = renderHook(() =>
      useAsyncResource(fetcher, [], { onError })
    );

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.data).toBeNull();
    expect(result.current.error).toBe("Network failure");
    expect(onError).toHaveBeenCalled();
  });

  it("re-fetches when refresh is called", async () => {
    let count = 0;
    const fetcher = vi.fn().mockImplementation(async () => {
      count += 1;
      return { count };
    });

    const { result } = renderHook(() => useAsyncResource(fetcher));

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });
    expect(result.current.data).toEqual({ count: 1 });

    await act(async () => {
      await result.current.refresh();
    });

    expect(result.current.data).toEqual({ count: 2 });
    expect(fetcher).toHaveBeenCalledTimes(2);
  });
});
