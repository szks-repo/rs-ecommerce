export const STOREFRONT_BASE =
  process.env.NEXT_PUBLIC_STOREFRONT_BASE || "http://localhost:3001";

export function buildProductPreviewUrl(productId: string) {
  if (!productId) {
    return STOREFRONT_BASE;
  }
  return `${STOREFRONT_BASE}/products/${productId}`;
}
