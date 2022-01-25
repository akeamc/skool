export async function retryRequest(
  promise: Promise<Response>,
  attempts = 5,
  backoff = 1000
) {
  let error: Error | null = null;

  while (attempts--) {
    try {
      const res = await promise;
      return res;
    } catch (e) {
      console.error(e);
      error = e as any;
    }
    await new Promise((resolve) => setTimeout(resolve, backoff));
  }

  throw error;
}
