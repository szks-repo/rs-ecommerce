export const SKU_CODE_REGEX_KEY = "store_sku_code_regex";

export function getSkuCodeRegex(): string | null {
  if (typeof window === "undefined") {
    return null;
  }
  const raw = window.sessionStorage.getItem(SKU_CODE_REGEX_KEY);
  if (!raw) {
    return null;
  }
  const trimmed = raw.trim();
  return trimmed.length > 0 ? trimmed : null;
}

export function validateSkuCode(sku: string): string | null {
  const rule = getSkuCodeRegex();
  if (!rule) {
    return null;
  }
  try {
    const regex = new RegExp(rule);
    if (!regex.test(sku)) {
      return "SKU format does not match store settings.";
    }
  } catch {
    return "SKU validation rule is invalid. Please update store settings.";
  }
  return null;
}
