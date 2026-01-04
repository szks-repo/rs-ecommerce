import type { Category } from "@/gen/ecommerce/v1/backoffice_pb";

export type FlattenedCategory = Category & {
  depth: number;
};

export function flattenCategories(categories: Category[]): FlattenedCategory[] {
  const byParent = new Map<string, Category[]>();
  for (const category of categories) {
    const key = category.parentId || "";
    const list = byParent.get(key);
    if (list) {
      list.push(category);
    } else {
      byParent.set(key, [category]);
    }
  }
  const sortByPosition = (items: Category[]) =>
    items.sort((a, b) => (a.position ?? 0) - (b.position ?? 0));

  const result: FlattenedCategory[] = [];
  const visit = (parentId: string, depth: number) => {
    const children = sortByPosition(byParent.get(parentId) ?? []);
    for (const child of children) {
      result.push({ ...child, depth });
      visit(child.id, depth + 1);
    }
  };
  visit("", 0);
  return result;
}

export function categoryLabel(category: FlattenedCategory): string {
  const level = `L${category.depth + 1}`;
  const prefix = category.depth > 0 ? "—".repeat(category.depth) + " " : "";
  return `${prefix}${category.name} (${level})`;
}
