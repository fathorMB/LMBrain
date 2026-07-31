import { describe, expect, it, vi } from "vitest";
import { createTrailingRefreshCoordinator } from "../lib/refreshCoordinator";

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, resolve, reject };
}

describe("createTrailingRefreshCoordinator", () => {
  it("coalesces an active burst into one trailing refresh and returns the newest snapshot", async () => {
    const first = deferred<string>();
    const second = deferred<string>();
    const load = vi
      .fn<() => Promise<string>>()
      .mockReturnValueOnce(first.promise)
      .mockReturnValueOnce(second.promise);
    const refresh = createTrailingRefreshCoordinator(load);

    const requestA = refresh();
    const requestB = refresh();
    const requestC = refresh();

    expect(load).toHaveBeenCalledTimes(1);
    expect(requestB).toBe(requestA);
    expect(requestC).toBe(requestA);

    first.resolve("stale");
    await first.promise;
    await vi.waitFor(() => expect(load).toHaveBeenCalledTimes(2));

    second.resolve("latest");
    await expect(requestA).resolves.toBe("latest");
    await expect(requestB).resolves.toBe("latest");
  });

  it("does not retry a failed refresh automatically and accepts a later explicit request", async () => {
    const failure = deferred<string>();
    const load = vi
      .fn<() => Promise<string>>()
      .mockReturnValueOnce(failure.promise)
      .mockResolvedValueOnce("recovered");
    const refresh = createTrailingRefreshCoordinator(load);

    const failedRequest = refresh();
    const coalescedRequest = refresh();
    failure.reject(new Error("backend unavailable"));

    await expect(failedRequest).rejects.toThrow("backend unavailable");
    await expect(coalescedRequest).rejects.toThrow("backend unavailable");
    expect(load).toHaveBeenCalledTimes(1);

    await expect(refresh()).resolves.toBe("recovered");
    expect(load).toHaveBeenCalledTimes(2);
  });
});
