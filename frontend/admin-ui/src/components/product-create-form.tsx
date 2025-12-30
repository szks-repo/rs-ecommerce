"use client";

import { useEffect, useState } from "react";
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
import { formatConnectError } from "@/lib/handle-error";

export default function ProductCreateForm() {
  const [title, setTitle] = useState("");
  const [vendorId, setVendorId] = useState("");
  const [description, setDescription] = useState("");
  const [status, setStatus] = useState("active");
  const [taxRuleId, setTaxRuleId] = useState("__default__");
  const [variantAxes, setVariantAxes] = useState("");
  const [sku, setSku] = useState("");
  const [fulfillmentType, setFulfillmentType] = useState("physical");
  const [priceAmount, setPriceAmount] = useState("0");
  const [compareAtAmount, setCompareAtAmount] = useState("");
  const [variantStatus, setVariantStatus] = useState("active");
  const [taxRules, setTaxRules] = useState<TaxRule[]>([]);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const { push } = useToast();
  const statusOptions = ["active", "inactive", "draft"] as const;
  const variantStatusOptions = ["active", "inactive"] as const;
  const fulfillmentOptions = ["physical", "digital"] as const;

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

  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setIsSubmitting(true);
    try {
      if (!getActiveAccessToken()) {
        throw new Error("access_token is missing. Please sign in first.");
      }
      const axes = variantAxes
        .split(",")
        .map((axis) => axis.trim())
        .filter((axis) => axis.length > 0);
      let defaultVariant = undefined as
        | {
            sku: string;
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
        const compareAt =
          compareAtAmount.trim().length > 0 ? Number(compareAtAmount) : undefined;
        if (typeof compareAt === "number" && !Number.isFinite(compareAt)) {
          throw new Error("compare_at_amount must be a number.");
        }
        defaultVariant = {
          sku: sku.trim(),
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
      setTitle("");
      setVendorId("");
      setDescription("");
      setStatus("active");
      setTaxRuleId("__default__");
      setVariantAxes("");
      setSku("");
      setFulfillmentType("physical");
      setPriceAmount("0");
      setCompareAtAmount("");
      setVariantStatus("active");
    } catch (err) {
      const uiError = formatConnectError(err, "Create failed", "Unknown error");
      push({
        variant: "error",
        title: uiError.title,
        description: uiError.description,
      });
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
              required
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
          <div className="space-y-2">
            <Label htmlFor="productVariantAxes">Variant Axes (comma separated)</Label>
            <Input
              id="productVariantAxes"
              value={variantAxes}
              onChange={(e) => setVariantAxes(e.target.value)}
              placeholder="Size, Color"
            />
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
                  disabled={variantAxes.trim().length > 0}
                  required={variantAxes.trim().length === 0}
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="defaultFulfillment">Fulfillment Type</Label>
                <Select
                  value={fulfillmentType}
                  onValueChange={setFulfillmentType}
                  disabled={variantAxes.trim().length > 0}
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
                    disabled={variantAxes.trim().length > 0}
                    required={variantAxes.trim().length === 0}
                  />
                </div>
                <div className="space-y-2">
                  <Label htmlFor="defaultCompareAt">Compare-at Amount (JPY)</Label>
                  <Input
                    id="defaultCompareAt"
                    value={compareAtAmount}
                    onChange={(e) => setCompareAtAmount(e.target.value)}
                    disabled={variantAxes.trim().length > 0}
                  />
                </div>
              </div>
              <div className="space-y-2">
                <Label htmlFor="defaultVariantStatus">Variant Status</Label>
                <Select
                  value={variantStatus}
                  onValueChange={setVariantStatus}
                  disabled={variantAxes.trim().length > 0}
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
