"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { useToast } from "@/components/ui/toast";
import { Textarea } from "@/components/ui/textarea";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { createProduct } from "@/lib/product";
import { getActiveAccessToken } from "@/lib/auth";
import { listTaxRules } from "@/lib/store_settings";
import type { TaxRule } from "@/gen/ecommerce/v1/store_settings_pb";
import { useApiCall } from "@/lib/use-api-call";
import { dateInputToTimestamp } from "@/lib/time";
import { getSkuCodeRegex, validateSkuCode } from "@/lib/sku-code";
import { listCategoriesAdmin } from "@/lib/category";
import type { Category } from "@/gen/ecommerce/v1/backoffice_pb";
import { categoryLabel, flattenCategories } from "@/lib/category-utils";

export default function ProductCreateForm() {
  const router = useRouter();
  const [title, setTitle] = useState("");
  const [vendorId, setVendorId] = useState("");
  const [description, setDescription] = useState("");
  const [status, setStatus] = useState("active");
  const [taxRuleId, setTaxRuleId] = useState("__default__");
  const [saleStartDate, setSaleStartDate] = useState("");
  const [saleEndDate, setSaleEndDate] = useState("");
  const [variantAxes, setVariantAxes] = useState<string[]>([]);
  const [newAxis, setNewAxis] = useState("");
  const [draggingAxisIndex, setDraggingAxisIndex] = useState<number | null>(null);
  const [sku, setSku] = useState("");
  const [skuError, setSkuError] = useState<string | null>(null);
  const [janCode, setJanCode] = useState("");
  const [fulfillmentType, setFulfillmentType] = useState("physical");
  const [priceAmount, setPriceAmount] = useState("0");
  const [compareAtAmount, setCompareAtAmount] = useState("");
  const [variantStatus, setVariantStatus] = useState("active");
  const [taxRules, setTaxRules] = useState<TaxRule[]>([]);
  const [categories, setCategories] = useState<Category[]>([]);
  const [primaryCategoryId, setPrimaryCategoryId] = useState("");
  const [additionalCategoryIds, setAdditionalCategoryIds] = useState<string[]>([]);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const { push } = useToast();
  const { notifyError } = useApiCall();
  const statusOptions = ["active", "inactive", "draft"] as const;
  const variantStatusOptions = ["active", "inactive"] as const;
  const fulfillmentOptions = ["physical", "digital"] as const;
  const axes = variantAxes.map((axis) => axis.trim()).filter((axis) => axis.length > 0);

  function addAxis(value: string) {
    const trimmed = value.trim();
    if (!trimmed) {
      return;
    }
    if (variantAxes.some((axis) => axis.trim().toLowerCase() === trimmed.toLowerCase())) {
      push({
        variant: "error",
        title: "Axis already exists",
        description: "Please use a unique axis name.",
      });
      return;
    }
    setVariantAxes([...variantAxes, trimmed]);
    setNewAxis("");
  }

  useEffect(() => {
    if (!getActiveAccessToken()) {
      return;
    }
    listTaxRules()
      .then((data) => {
        setTaxRules(data.rules ?? []);
      })
      .catch(() => {
        setTaxRules([]);
      });
  }, []);

  useEffect(() => {
    if (!getActiveAccessToken()) {
      return;
    }
    listCategoriesAdmin()
      .then((data) => {
        setCategories(data.categories ?? []);
      })
      .catch(() => {
        setCategories([]);
      });
  }, []);

  useEffect(() => {
    if (!sku.trim()) {
      setSkuError(null);
      return;
    }
    setSkuError(validateSkuCode(sku.trim()));
  }, [sku]);

  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setIsSubmitting(true);
    try {
      if (!getActiveAccessToken()) {
        throw new Error("access_token is missing. Please sign in first.");
      }
      const saleStartAt = dateInputToTimestamp(saleStartDate, false);
      const saleEndAt = dateInputToTimestamp(saleEndDate, true);
      if (saleStartAt && saleEndAt && saleStartAt.seconds > saleEndAt.seconds) {
        throw new Error("sale_end_at must be later than sale_start_at.");
      }
      const normalizedPrimary =
        primaryCategoryId || additionalCategoryIds[0] || "";
      let defaultVariant = undefined as
        | {
            sku: string;
            janCode?: string;
            fulfillmentType: string;
            priceAmount: number;
            compareAtAmount?: number;
            currency: string;
            status: string;
          }
        | undefined;
      if (axes.length === 0) {
        const trimmedPrice = priceAmount.trim();
        if (trimmedPrice.length === 0) {
          throw new Error("price_amount is required when no variant axes are specified.");
        }
        const price = Number(trimmedPrice);
        if (!Number.isFinite(price)) {
          throw new Error("price_amount must be a number.");
        }
        if (!sku.trim()) {
          throw new Error("sku is required when no variant axes are specified.");
        }
        if (skuError) {
          throw new Error(skuError);
        }
        const compareAt =
          compareAtAmount.trim().length > 0 ? Number(compareAtAmount) : undefined;
        if (typeof compareAt === "number" && !Number.isFinite(compareAt)) {
          throw new Error("compare_at_amount must be a number.");
        }
        defaultVariant = {
          sku: sku.trim(),
          janCode: janCode.trim() || undefined,
          fulfillmentType,
          priceAmount: price,
          compareAtAmount: compareAt,
          currency: "JPY",
          status: variantStatus,
        };
      }
      const payload: Parameters<typeof createProduct>[0] = {
        vendorId: vendorId.trim() || undefined,
        title,
        description,
        status,
        taxRuleId: taxRuleId === "__default__" ? undefined : taxRuleId || undefined,
        saleStartAt,
        saleEndAt,
        primaryCategoryId: normalizedPrimary,
        categoryIds: normalizedPrimary
          ? [
              normalizedPrimary,
              ...additionalCategoryIds.filter((id) => id !== normalizedPrimary),
            ]
          : [],
        variantAxes: axes.map((name, index) => ({ name, position: index + 1 })),
      };
      if (defaultVariant) {
        payload.defaultVariant = defaultVariant;
      }
      const data = await createProduct(payload);
      push({
        variant: "success",
        title: "Product created",
        description: `Created product: ${data.product.id}`,
      });
      router.push(`/admin/products/${data.product.id}`);
      setTitle("");
      setVendorId("");
      setDescription("");
      setStatus("active");
      setTaxRuleId("__default__");
      setSaleStartDate("");
      setSaleEndDate("");
      setPrimaryCategoryId("");
      setAdditionalCategoryIds([]);
      setVariantAxes([]);
      setNewAxis("");
      setSku("");
      setJanCode("");
      setFulfillmentType("physical");
      setPriceAmount("0");
      setCompareAtAmount("");
      setVariantStatus("active");
    } catch (err) {
      notifyError(err, "Create failed", "Unknown error");
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <Card className="border-neutral-200 bg-white text-neutral-900">
      <CardHeader>
        <CardTitle>Create Product</CardTitle>
        <CardDescription className="text-neutral-500">
          Register product master data.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form className="grid gap-4" onSubmit={handleSubmit}>
          <div className="space-y-2">
            <Label htmlFor="productPrimaryCategory">Primary Category (optional)</Label>
            <Select value={primaryCategoryId} onValueChange={setPrimaryCategoryId}>
              <SelectTrigger id="productPrimaryCategory" className="bg-white">
                <SelectValue placeholder="Select primary category" />
              </SelectTrigger>
              <SelectContent>
                {flattenCategories(categories).map((category) => (
                  <SelectItem key={category.id} value={category.id}>
                    {categoryLabel(category)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            {categories.length === 0 && (
              <p className="text-xs text-neutral-500">
                No categories yet. Create one in the Categories menu.
              </p>
            )}
            <p className="text-xs text-neutral-500">
              Categories are optional. If you choose additional categories, the first one becomes
              the primary category unless you pick one here.
            </p>
          </div>
          <div className="space-y-2">
            <Label>Additional Categories (optional)</Label>
            <div className="grid gap-2 rounded-md border border-neutral-200 p-3 text-sm text-neutral-600">
              {flattenCategories(categories).map((category) => (
                <label key={category.id} className="flex items-center gap-2">
                  <input
                    type="checkbox"
                    className="h-4 w-4"
                    checked={additionalCategoryIds.includes(category.id)}
                    onChange={(event) => {
                      setAdditionalCategoryIds((prev) => {
                        if (event.target.checked) {
                          return [...prev, category.id];
                        }
                        return prev.filter((id) => id !== category.id);
                      });
                    }}
                  />
                  <span>{categoryLabel(category)}</span>
                </label>
              ))}
              {categories.length === 0 && (
                <span className="text-xs text-neutral-500">
                  Register categories to assign them here.
                </span>
              )}
            </div>
          </div>
          <div className="space-y-2">
            <Label htmlFor="productTitle">Title</Label>
            <Input
              id="productTitle"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              required
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="productVendor">Vendor ID (optional)</Label>
            <Input
              id="productVendor"
              value={vendorId}
              onChange={(e) => setVendorId(e.target.value)}
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="productDescription">Description</Label>
            <Textarea
              id="productDescription"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              rows={4}
            />
          </div>
          <div className="space-y-2">
            <Label htmlFor="productStatus">Status</Label>
            <Select value={status} onValueChange={setStatus}>
              <SelectTrigger id="productStatus" className="bg-white">
                <SelectValue placeholder="Select status" />
              </SelectTrigger>
              <SelectContent>
                {statusOptions.map((option) => (
                  <SelectItem key={option} value={option}>
                    {option}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-2">
            <Label htmlFor="productTaxRule">Tax Rule</Label>
            <Select value={taxRuleId} onValueChange={setTaxRuleId}>
              <SelectTrigger id="productTaxRule" className="bg-white">
                <SelectValue placeholder="Default (store setting)" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="__default__">Default (store setting)</SelectItem>
                {taxRules.map((rule) => (
                  <SelectItem key={rule.id} value={rule.id}>
                    {rule.name} ({(rule.rate * 100).toFixed(1)}%)
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="grid gap-4 md:grid-cols-2">
            <div className="space-y-2">
              <Label htmlFor="productSaleStart">Sale start date (optional)</Label>
              <Input
                id="productSaleStart"
                type="date"
                value={saleStartDate}
                onChange={(e) => setSaleStartDate(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="productSaleEnd">Sale end date (optional)</Label>
              <Input
                id="productSaleEnd"
                type="date"
                value={saleEndDate}
                onChange={(e) => setSaleEndDate(e.target.value)}
              />
            </div>
            <div className="text-xs text-neutral-500 md:col-span-2">
              Set either start or end date (or both). Leave empty to keep it always purchasable.
            </div>
          </div>
          <div className="space-y-2">
            <Label htmlFor="productVariantAxes">Variant Axes</Label>
            <div className="space-y-3">
              <div className="flex flex-wrap gap-2">
                {axes.length === 0 ? (
                  <span className="rounded-full border border-dashed border-neutral-200 px-3 py-1 text-xs text-neutral-400">
                    No axes yet
                  </span>
                ) : (
                  axes.map((axis, index) => (
                    <div
                      key={`${axis}-${index}`}
                      className="flex items-center gap-2 rounded-full border border-neutral-200 bg-white px-3 py-1 text-xs text-neutral-700 shadow-sm"
                      draggable
                      onDragStart={() => setDraggingAxisIndex(index)}
                      onDragOver={(event) => event.preventDefault()}
                      onDrop={() => {
                        if (draggingAxisIndex === null || draggingAxisIndex === index) {
                          setDraggingAxisIndex(null);
                          return;
                        }
                        const next = [...axes];
                        const [moved] = next.splice(draggingAxisIndex, 1);
                        next.splice(index, 0, moved);
                        setVariantAxes(next);
                        setDraggingAxisIndex(null);
                      }}
                    >
                      <span className="cursor-grab text-neutral-400">⋮⋮</span>
                      <span>{axis}</span>
                      <button
                        type="button"
                        className="text-neutral-400 hover:text-neutral-700"
                        onClick={() => {
                          const next = axes.filter((_, idx) => idx !== index);
                          setVariantAxes(next);
                        }}
                      >
                        ×
                      </button>
                    </div>
                  ))
                )}
              </div>
              <div className="flex flex-col gap-2 md:flex-row md:items-center">
                <Input
                  id="productVariantAxes"
                  value={newAxis}
                  onChange={(e) => setNewAxis(e.target.value)}
                  placeholder="Add axis (e.g. Size)"
                  onKeyDown={(event) => {
                    if (event.key === "Enter") {
                      event.preventDefault();
                      addAxis(newAxis);
                    }
                  }}
                />
                <Button type="button" variant="outline" onClick={() => addAxis(newAxis)}>
                  Add axis
                </Button>
              </div>
              <div className="text-xs text-neutral-500">
                Drag chips to reorder axes. The order affects SKU creation.
              </div>
            </div>
            <p className="text-xs text-neutral-500">
              If you set axes, the default SKU will not be created automatically.
            </p>
          </div>
          <div className="rounded-md border border-neutral-200 bg-neutral-50 p-4">
            <div className="text-sm font-semibold text-neutral-700">Default SKU</div>
            <div className="text-xs text-neutral-500">
              Required when no variant axes are specified.
            </div>
            <div className="mt-4 grid gap-4">
              <div className="space-y-2">
                <Label htmlFor="defaultSku">SKU</Label>
                <Input
                  id="defaultSku"
                  value={sku}
                  onChange={(e) => setSku(e.target.value)}
                  disabled={axes.length > 0}
                  required={axes.length === 0}
                />
                {skuError && axes.length === 0 ? (
                  <p className="text-xs text-red-600">{skuError}</p>
                ) : null}
                {getSkuCodeRegex() && !skuError && axes.length === 0 ? (
                  <p className="text-xs text-neutral-500">
                    Rule: {getSkuCodeRegex()}
                  </p>
                ) : null}
              </div>
              <div className="space-y-2">
                <Label htmlFor="defaultJanCode">JAN Code (optional)</Label>
                <Input
                  id="defaultJanCode"
                  value={janCode}
                  onChange={(e) => setJanCode(e.target.value)}
                  disabled={axes.length > 0}
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="defaultFulfillment">Fulfillment Type</Label>
                <Select
                  value={fulfillmentType}
                  onValueChange={setFulfillmentType}
                  disabled={axes.length > 0}
                >
                  <SelectTrigger id="defaultFulfillment" className="bg-white">
                    <SelectValue placeholder="Select fulfillment type" />
                  </SelectTrigger>
                  <SelectContent>
                    {fulfillmentOptions.map((option) => (
                      <SelectItem key={option} value={option}>
                        {option}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="grid gap-4 md:grid-cols-2">
                <div className="space-y-2">
                  <Label htmlFor="defaultPrice">Price Amount (JPY)</Label>
                  <Input
                    id="defaultPrice"
                    value={priceAmount}
                    onChange={(e) => setPriceAmount(e.target.value)}
                    disabled={axes.length > 0}
                    required={axes.length === 0}
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="defaultCompareAt">Compare-at Amount (JPY)</Label>
                  <Input
                    id="defaultCompareAt"
                    value={compareAtAmount}
                    onChange={(e) => setCompareAtAmount(e.target.value)}
                    disabled={axes.length > 0}
                  />
                </div>
              </div>
              <div className="space-y-2">
                <Label htmlFor="defaultVariantStatus">Variant Status</Label>
                <Select
                  value={variantStatus}
                  onValueChange={setVariantStatus}
                  disabled={axes.length > 0}
                >
                  <SelectTrigger id="defaultVariantStatus" className="bg-white">
                    <SelectValue placeholder="Select status" />
                  </SelectTrigger>
                  <SelectContent>
                    {variantStatusOptions.map((option) => (
                      <SelectItem key={option} value={option}>
                        {option}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>
          </div>
          <div>
            <Button type="submit" disabled={isSubmitting}>
              {isSubmitting ? "Creating..." : "Create Product"}
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  );
}
