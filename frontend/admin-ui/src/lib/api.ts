const DEFAULT_API_BASE = "http://localhost:8080/rpc";

export const API_BASE =
  process.env.NEXT_PUBLIC_API_BASE ||
  (typeof window !== "undefined"
    ? `http://${window.location.hostname}:8080/rpc`
    : DEFAULT_API_BASE);
