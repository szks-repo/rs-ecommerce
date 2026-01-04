export type Brand<T, BrandName extends string> = T & {
  readonly __brand: BrandName;
};

export function brand<T, BrandName extends string>(value: T): Brand<T, BrandName> {
  return value as Brand<T, BrandName>;
}
