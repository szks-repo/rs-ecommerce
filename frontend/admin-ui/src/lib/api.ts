export const API_BASE = process.env.NEXT_PUBLIC_API_BASE || "http://localhost:8080";

export type ApiError = {
  code?: string;
  message?: string;
};

export async function rpcFetch<T>(path: string, body: unknown): Promise<T> {
  const token = typeof window !== "undefined" ? sessionStorage.getItem("access_token") : null;
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
  };
  if (token) {
    headers.Authorization = `Bearer ${token}`;
  }

  const res = await fetch(`${API_BASE}${path}`, {
    method: "POST",
    headers,
    body: JSON.stringify(body),
  });

  if (!res.ok) {
    const data = (await res.json().catch(() => null)) as ApiError | null;
    const message = data?.message || `Request failed (${res.status})`;
    throw new Error(message);
  }

  return res.json();
}
