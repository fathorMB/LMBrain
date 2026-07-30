export type RefreshRequest<T> = () => Promise<T>;

/**
 * Serializes refresh work and collapses any number of requests received while
 * a load is active into one trailing load. Every caller observes the newest
 * completed value, so an older snapshot can never overwrite a newer one.
 */
export function createTrailingRefreshCoordinator<T>(
  load: RefreshRequest<T>,
): RefreshRequest<T> {
  let inFlight: Promise<T> | null = null;
  let trailingRefreshRequested = false;

  return () => {
    if (inFlight) {
      trailingRefreshRequested = true;
      return inFlight;
    }

    const run = async (): Promise<T> => {
      let latest: T;
      do {
        trailingRefreshRequested = false;
        latest = await load();
      } while (trailingRefreshRequested);
      return latest;
    };

    const request = run().finally(() => {
      if (inFlight === request) {
        inFlight = null;
        trailingRefreshRequested = false;
      }
    });
    inFlight = request;
    return request;
  };
}
