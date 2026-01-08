"use client";

import { useEffect, useMemo, useState } from "react";
import Link from "next/link";
import { useParams } from "next/navigation";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { useToast } from "@/components/ui/toast";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import ProductUpdateForm from "@/components/product-update-form";
import VariantUpdateForm from "@/components/variant-update-form";
import VariantCreateForm from "@/components/variant-create-form";
import VariantBulkUpdateForm from "@/components/variant-bulk-update-form";
import SkuImageManager from "@/components/sku-image-manager";
import SkuDigitalAssetManager from "@/components/sku-digital-asset-manager";
import ProductInventoryPanel from "@/components/product-inventory-panel";
import {
  listProductsAdmin,
  listVariantsAdmin,
  listProductMetafieldDefinitions,
  listProductMetafieldValues,
  upsertProductMetafieldValue,
} from "@/lib/product";
import { getActiveAccessToken } from "@/lib/auth";
import { buildProductPreviewUrl } from "@/lib/storefront";
import type {
  ProductMetafieldDefinition,
  ProductAdmin,
  VariantAdmin,
} from "@/gen/ecommerce/v1/backoffice_pb";
import { useApiCall } from "@/lib/use-api-call";
import AdminPageHeader from "@/components/admin-page-header";
import { formatTimestampWithStoreTz } from "@/lib/time";

type MetafieldValueState = string | string[] | boolean;

function normalizeMetafieldValue(valueJson?: string): MetafieldValueState {
  if (!valueJson) {
    return "";
  }
  try {
    const parsed = JSON.parse(valueJson);
    if (Array.isArray(parsed)) {
      return parsed.map((item) => String(item));
    }
    if (typeof parsed === "boolean") {
      return parsed;
    }
    if (typeof parsed === "string") {
      return parsed;
    }
    if (typeof parsed === "number") {
      return String(parsed);
    }
    return JSON.stringify(parsed);
  } catch {
    return valueJson;
  }
}

export default function ProductDetailPage() {
  const params = useParams();
  const productId = useMemo(() => {
    if (!params?.productId) {
      return "";
    }
    return Array.isArray(params.productId) ? params.productId[0] : params.productId;
  }, [params]);

  const [product, setProduct] = useState<ProductAdmin | null>(null);
  const [variants, setVariants] = useState<VariantAdmin[]>([]);
  const [selectedVariant, setSelectedVariant] = useState<VariantAdmin | null>(null);
  const [variantAxes, setVariantAxes] = useState<{ name: string; position?: number }[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [metafieldDefs, setMetafieldDefs] = useState<ProductMetafieldDefinition[]>([]);
  const [metafieldValues, setMetafieldValues] = useState<Record<string, MetafieldValueState>>(
    {}
  );
  const [isSavingMetafield, setIsSavingMetafield] = useState<string | null>(null);
  const { push } = useToast();
  const { notifyError } = useApiCall();

  async function loadData() {
    if (!getActiveAccessToken()) {
      push({
        variant: "error",
        title: "Load failed",
        description: "access_token is missing. Please sign in first.",
      });
      return;
    }
    if (!productId) {
      push({
        variant: "error",
        title: "Load failed",
        description: "product_id is missing.",
      });
      return;
    }
    setIsLoading(true);
    try {
      const [productResp, variantsResp, defsResp, valuesResp] = await Promise.all([
        listProductsAdmin(),
        listVariantsAdmin({ productId }),
        listProductMetafieldDefinitions(),
        listProductMetafieldValues(productId),
      ]);
      const found = (productResp.products ?? []).find((p) => p.id === productId) ?? null;
      setProduct(found);
      setVariants(variantsResp.variants ?? []);
      setVariantAxes(variantsResp.variantAxes ?? []);
      setMetafieldDefs(defsResp.definitions ?? []);
      const valuesMap: Record<string, MetafieldValueState> = {};
      (valuesResp.values ?? []).forEach((value) => {
        if (value.definitionId) {
          valuesMap[value.definitionId] = normalizeMetafieldValue(value.valueJson);
        }
      });
      setMetafieldValues(valuesMap);
      if (variantsResp.variants && variantsResp.variants.length > 0) {
        const next = selectedVariant
          ? variantsResp.variants.find((v) => v.id === selectedVariant.id) ?? variantsResp.variants[0]
          : variantsResp.variants[0];
        setSelectedVariant(next ?? null);
      } else {
        setSelectedVariant(null);
      }
    } catch (err) {
      notifyError(err, "Load failed", "Failed to load product");
    } finally {
      setIsLoading(false);
    }
  }

  useEffect(() => {
    void loadData();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [productId]);

  function validateMetafieldValue(
    definition: ProductMetafieldDefinition,
    rawValue: MetafieldValueState
  ) {
    let validations: { min?: unknown; max?: unknown; required?: unknown; regex?: unknown } = {};
    if (definition.validationsJson) {
      try {
        validations = JSON.parse(definition.validationsJson);
      } catch {
        validations = {};
      }
    }
    const required = validations.required === true;
    const isDateType = definition.valueType === "date" || definition.valueType === "dateTime";
    const isBooleanType = definition.valueType === "bool" || definition.valueType === "boolean";
    const isNumberType = definition.valueType === "number";

    const isEmpty =
      rawValue == null ||
      (typeof rawValue === "string" && rawValue.trim() === "") ||
      (Array.isArray(rawValue) && rawValue.length === 0);

    if (required && isEmpty) {
      return "This field is required.";
    }

    if (isBooleanType) {
      return null;
    }

    if (isDateType) {
      const minValue = typeof validations.min === "string" ? validations.min : undefined;
      const maxValue = typeof validations.max === "string" ? validations.max : undefined;
      const parseDate = (value: string) => {
        if (definition.valueType === "date") {
          return new Date(`${value}T00:00:00`);
        }
        return new Date(value);
      };
      const minDate = minValue ? parseDate(minValue) : null;
      const maxDate = maxValue ? parseDate(maxValue) : null;
      const values: string[] = [];
      if (definition.isList) {
        if (Array.isArray(rawValue)) {
          values.push(...rawValue.map((v) => String(v)));
        } else if (typeof rawValue === "string" && rawValue.trim() !== "") {
          try {
            const parsed = JSON.parse(rawValue);
            if (!Array.isArray(parsed)) {
              return "Value must be a JSON array of date strings.";
            }
            values.push(...parsed.map((v: unknown) => String(v)));
          } catch {
            return "Value must be a JSON array of date strings.";
          }
        }
      } else if (typeof rawValue === "string" && rawValue.trim() !== "") {
        values.push(rawValue.trim());
      }

      if (values.length === 0) {
        return null;
      }

      for (const value of values) {
        const date = parseDate(value);
        if (!Number.isFinite(date.getTime())) {
          return "Value must be a valid date.";
        }
        if (minDate && Number.isFinite(minDate.getTime()) && date < minDate) {
          return `Date must be on or after ${minValue}.`;
        }
        if (maxDate && Number.isFinite(maxDate.getTime()) && date > maxDate) {
          return `Date must be on or before ${maxValue}.`;
        }
      }
    }

    if (isNumberType) {
      const minValue = typeof validations.min === "number" ? validations.min : undefined;
      const maxValue = typeof validations.max === "number" ? validations.max : undefined;
      const values: number[] = [];
      if (definition.isList) {
        if (Array.isArray(rawValue)) {
          values.push(...rawValue.map((v) => Number(v)));
        } else if (typeof rawValue === "string" && rawValue.trim() !== "") {
          try {
            const parsed = JSON.parse(rawValue);
            if (!Array.isArray(parsed)) {
              return "Value must be a JSON array of numbers.";
            }
            values.push(...parsed.map((v: unknown) => Number(v)));
          } catch {
            return "Value must be a JSON array of numbers.";
          }
        }
      } else if (typeof rawValue === "string" && rawValue.trim() !== "") {
        values.push(Number(rawValue.trim()));
      }

      if (values.length === 0) {
        return null;
      }

      for (const num of values) {
        if (!Number.isFinite(num)) {
          return "Value must be a number.";
        }
        if (minValue != null && num < minValue) {
          return `Value must be at least ${minValue}.`;
        }
        if (maxValue != null && num > maxValue) {
          return `Value must be at most ${maxValue}.`;
        }
      }
    }

    if (
      definition.valueType === "string" ||
      definition.valueType === "text" ||
      definition.valueType === "json"
    ) {
      const minValue = typeof validations.min === "number" ? validations.min : undefined;
      const maxValue = typeof validations.max === "number" ? validations.max : undefined;
      const regexValue = typeof validations.regex === "string" ? validations.regex : undefined;
      let regex: RegExp | null = null;
      if (regexValue) {
        try {
          regex = new RegExp(regexValue);
        } catch {
          regex = null;
        }
      }
      const values: string[] = [];
      if (definition.isList) {
        if (Array.isArray(rawValue)) {
          values.push(...rawValue.map((v) => String(v)));
        } else if (typeof rawValue === "string" && rawValue.trim() !== "") {
          try {
            const parsed = JSON.parse(rawValue);
            if (!Array.isArray(parsed)) {
              return "Value must be a JSON array of strings.";
            }
            values.push(...parsed.map((v: unknown) => String(v)));
          } catch {
            return "Value must be a JSON array of strings.";
          }
        }
      } else if (typeof rawValue === "string" && rawValue.trim() !== "") {
        values.push(rawValue.trim());
      }

      if (values.length === 0) {
        return null;
      }

      for (const text of values) {
        if (minValue != null && text.length < minValue) {
          return `Value must be at least ${minValue} characters.`;
        }
        if (maxValue != null && text.length > maxValue) {
          return `Value must be at most ${maxValue} characters.`;
        }
        if (regex && !regex.test(text)) {
          return "Value does not match the required pattern.";
        }
      }
    }

    return null;
  }

  async function handleSaveMetafield(definition: ProductMetafieldDefinition) {
    if (!definition.id || isSavingMetafield) {
      return;
    }
    setIsSavingMetafield(definition.id);
    try {
      const rawValue = metafieldValues[definition.id];
      const validationError = validateMetafieldValue(definition, rawValue);
      if (validationError) {
        push({
          variant: "error",
          title: "Validation failed",
          description: validationError,
        });
        return;
      }

      let valueJson = "\"\"";
      const isBooleanType =
        definition.valueType === "bool" || definition.valueType === "boolean";
      if (isBooleanType) {
        const normalized =
          typeof rawValue === "string"
            ? rawValue.trim().toLowerCase() === "true"
            : Boolean(rawValue);
        valueJson = JSON.stringify(normalized);
      } else if (definition.isList) {
        if (Array.isArray(rawValue)) {
          valueJson = JSON.stringify(rawValue);
        } else if (typeof rawValue === "string" && rawValue.trim() !== "") {
          valueJson = rawValue.trim();
        } else {
          valueJson = "[]";
        }
      } else {
        valueJson = JSON.stringify(typeof rawValue === "string" ? rawValue : "");
      }
      await upsertProductMetafieldValue({
        productId,
        definitionId: definition.id,
        valueJson,
      });
      push({
        variant: "success",
        title: "Custom attribute saved",
        description: "Metafield value has been updated.",
      });
      await loadData();
    } catch (err) {
      notifyError(err, "Save failed", "Failed to save custom attribute");
    } finally {
      setIsSavingMetafield(null);
    }
  }

  function renderMetafieldInput(definition: ProductMetafieldDefinition) {
    let value: MetafieldValueState = metafieldValues[definition.id] ?? "";
    if (
      (definition.valueType === "bool" || definition.valueType === "boolean") &&
      typeof value === "string"
    ) {
      if (value.toLowerCase() === "true") {
        value = true;
      } else if (value.toLowerCase() === "false") {
        value = false;
      }
    }
    const handleChange = (nextValue: MetafieldValueState) => {
      setMetafieldValues((prev) => ({
        ...prev,
        [definition.id]: nextValue,
      }));
    };

    const validationError = validateMetafieldValue(definition, value);
    let enumOptions: string[] = [];
    if (definition.valueType === "enum") {
      try {
        const parsed = definition.validationsJson
          ? JSON.parse(definition.validationsJson)
          : {};
        if (parsed && Array.isArray(parsed.enum)) {
          enumOptions = parsed.enum.map((item: unknown) => String(item));
        }
      } catch {
        enumOptions = [];
      }
    }

    if (definition.valueType === "enum") {
      if (definition.isList) {
        const selected = Array.isArray(value) ? value : [];
        return (
          <div className="space-y-2">
            {enumOptions.length === 0 ? (
              <div className="text-xs text-neutral-500">No enum options configured.</div>
            ) : (
              <div className="flex flex-wrap gap-2">
                {enumOptions.map((option) => {
                  const checked = selected.includes(option);
                  return (
                    <label
                      key={option}
                      className="flex items-center gap-2 rounded-full border border-neutral-200 px-3 py-1 text-xs text-neutral-700"
                    >
                      <input
                        type="checkbox"
                        className="h-4 w-4 accent-neutral-900"
                        checked={checked}
                        onChange={(event) => {
                          if (event.target.checked) {
                            handleChange([...selected, option]);
                          } else {
                            handleChange(selected.filter((item) => item !== option));
                          }
                        }}
                      />
                      {option}
                    </label>
                  );
                })}
              </div>
            )}
            {validationError ? (
              <p className="text-xs text-red-600">{validationError}</p>
            ) : null}
          </div>
        );
      }
      return (
        <div className="space-y-2">
          <Select
            value={typeof value === "string" ? value : ""}
            onValueChange={(next) => handleChange(next)}
          >
            <SelectTrigger className="bg-white">
              <SelectValue placeholder="Select value" />
            </SelectTrigger>
            <SelectContent>
              {enumOptions.map((option) => (
                <SelectItem key={option} value={option}>
                  {option}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          {validationError ? (
            <p className="text-xs text-red-600">{validationError}</p>
          ) : null}
        </div>
      );
    }

    if (definition.valueType === "bool" || definition.valueType === "boolean") {
      return (
        <div className="flex items-center justify-between gap-3">
          <div className="text-xs text-neutral-500">Toggle on/off</div>
          <Switch checked={Boolean(value)} onCheckedChange={(checked) => handleChange(checked)} />
          {validationError ? <p className="text-xs text-red-600">{validationError}</p> : null}
        </div>
      );
    }

    if (definition.isList) {
      return (
        <div className="space-y-2">
          <Textarea
            id={`metafield-${definition.id}`}
            value={typeof value === "string" ? value : JSON.stringify(value)}
            onChange={(event) => handleChange(event.target.value)}
            placeholder='e.g. ["value1","value2"]'
          />
          {validationError ? <p className="text-xs text-red-600">{validationError}</p> : null}
        </div>
      );
    }

    switch (definition.valueType) {
      case "date":
        return (
          <div className="space-y-2">
            <Input
              id={`metafield-${definition.id}`}
              type="date"
              value={typeof value === "string" ? value : ""}
              onChange={(event) => handleChange(event.target.value)}
            />
            {validationError ? <p className="text-xs text-red-600">{validationError}</p> : null}
          </div>
        );
      case "dateTime":
        return (
          <div className="space-y-2">
            <Input
              id={`metafield-${definition.id}`}
              type="datetime-local"
              value={typeof value === "string" ? value : ""}
              onChange={(event) => handleChange(event.target.value)}
            />
            {validationError ? <p className="text-xs text-red-600">{validationError}</p> : null}
          </div>
        );
      case "color":
        return (
          <div className="flex items-center gap-3">
            <Input
              id={`metafield-${definition.id}`}
              type="color"
              value={typeof value === "string" && value ? value : "#000000"}
              onChange={(event) => handleChange(event.target.value)}
              className="h-10 w-16 p-1"
            />
            <Input
              value={typeof value === "string" ? value : ""}
              onChange={(event) => handleChange(event.target.value)}
              placeholder="#000000"
            />
            {validationError ? <p className="text-xs text-red-600">{validationError}</p> : null}
          </div>
        );
      default:
        return (
          <div className="space-y-2">
            <Input
              id={`metafield-${definition.id}`}
              value={typeof value === "string" ? value : ""}
              onChange={(event) => handleChange(event.target.value)}
              placeholder="Enter value"
            />
            {validationError ? <p className="text-xs text-red-600">{validationError}</p> : null}
          </div>
        );
    }
  }

  return (
    <div className="space-y-8">
      <AdminPageHeader
        title={product?.title || "Product details"}
        description="Review the product and manage its SKUs."
        actions={
          <>
            <a
              className="rounded-md border border-neutral-200 px-3 py-2 text-sm text-neutral-700 hover:bg-neutral-50"
              href={buildProductPreviewUrl(productId)}
              target="_blank"
              rel="noreferrer"
            >
              Preview
            </a>
            <Link
              className="rounded-md border border-neutral-200 px-3 py-2 text-sm text-neutral-700 hover:bg-neutral-50"
              href="/admin/products"
            >
              Back
            </Link>
            <Button variant="outline" onClick={loadData} disabled={isLoading}>
              {isLoading ? "Loading..." : "Refresh"}
            </Button>
          </>
        }
      />

      <Card className="border-neutral-200 bg-white text-neutral-900">
        <CardHeader>
          <CardTitle>Product</CardTitle>
          <CardDescription className="text-neutral-500">
            {product ? `status: ${product.status}` : "Select a product to review."}
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3 text-sm text-neutral-700">
          {product ? (
            <>
              <div className="text-xs text-neutral-500">id: {product.id}</div>
              <div className="text-xs text-neutral-500">
                tax rule: {product.taxRuleId || "default"}
              </div>
              <div className="text-xs text-neutral-500">
                sale period:{" "}
                {product.saleStartAt
                  ? formatTimestampWithStoreTz(
                      product.saleStartAt.seconds,
                      product.saleStartAt.nanos
                    )
                  : "always"}{" "}
                -{" "}
                {product.saleEndAt
                  ? formatTimestampWithStoreTz(
                      product.saleEndAt.seconds,
                      product.saleEndAt.nanos
                    )
                  : "always"}
              </div>
              {product.description ? <div>{product.description}</div> : <div>No description.</div>}
            </>
          ) : (
            <div className="text-sm text-neutral-600">Product not found in recent list.</div>
          )}
        </CardContent>
      </Card>

      <Card className="border-neutral-200 bg-white text-neutral-900">
        <CardHeader>
          <CardTitle>Metafields</CardTitle>
          <CardDescription className="text-neutral-500">
            Custom attributes attached to this product.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {metafieldDefs.length === 0 ? (
            <div className="text-sm text-neutral-600">No metafield definitions yet.</div>
          ) : (
            metafieldDefs.map((definition) => (
              <div
                key={definition.id}
                className="rounded-lg border border-neutral-200 p-3"
              >
                <div className="flex flex-wrap items-start justify-between gap-3">
                  <div>
                    <div className="text-sm font-medium text-neutral-900">{definition.name}</div>
                    <div className="text-xs text-neutral-500">
                      {definition.namespace}.{definition.key} · type: {definition.valueType}
                      {definition.isList ? " (list)" : ""}
                    </div>
                  </div>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => handleSaveMetafield(definition)}
                    disabled={
                      isSavingMetafield === definition.id ||
                      Boolean(validateMetafieldValue(definition, metafieldValues[definition.id]))
                    }
                  >
                    {isSavingMetafield === definition.id ? "Saving..." : "Save"}
                  </Button>
                </div>
                <div className="mt-3">{renderMetafieldInput(definition)}</div>
              </div>
            ))
          )}
        </CardContent>
      </Card>

      <div className="grid gap-6 md:grid-cols-2">
        <ProductUpdateForm product={product} onUpdated={loadData} />
        <VariantCreateForm
          productId={productId}
          variantAxes={variantAxes}
          onCreated={loadData}
        />
      </div>

      <Card className="border-neutral-200 bg-white text-neutral-900">
        <CardHeader>
          <CardTitle>SKUs</CardTitle>
          <CardDescription className="text-neutral-500">
            Select a variant to edit.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3 text-sm text-neutral-700">
          {variants.length === 0 ? (
            <div className="text-sm text-neutral-600">No variants yet.</div>
          ) : (
            variants.map((variant) => (
              <button
                key={variant.id}
                className={`w-full rounded-lg border px-3 py-2 text-left transition ${
                  selectedVariant?.id === variant.id
                    ? "border-neutral-900 bg-neutral-50"
                    : "border-neutral-200 hover:bg-neutral-50"
                }`}
                type="button"
                onClick={() => setSelectedVariant(variant)}
              >
                <div className="font-medium text-neutral-900">{variant.sku}</div>
                <div className="text-xs text-neutral-500">
                  id: {variant.id} / status: {variant.status} / type: {variant.fulfillmentType}
                </div>
                {variant.axisValues && variant.axisValues.length > 0 ? (
                  <div className="mt-1 text-xs text-neutral-600">
                    {variant.axisValues
                      .map((axis) => `${axis.name}: ${axis.value}`)
                      .join(" / ")}
                  </div>
                ) : null}
              </button>
            ))
          )}
        </CardContent>
      </Card>

      <div className="grid gap-6 md:grid-cols-2">
        <VariantUpdateForm
          variant={selectedVariant}
          variantAxes={variantAxes}
          onUpdated={loadData}
        />
        <VariantBulkUpdateForm variants={variants} onUpdated={loadData} />
      </div>

      {selectedVariant ? (
        <div className="space-y-6">
          <ProductInventoryPanel skuId={selectedVariant.id} skuLabel={selectedVariant.sku} />
          <SkuImageManager skuId={selectedVariant.id} />
          {selectedVariant.fulfillmentType === "digital" ? (
            <SkuDigitalAssetManager skuId={selectedVariant.id} />
          ) : null}
        </div>
      ) : (
        <Card className="border-neutral-200 bg-white text-neutral-900">
          <CardHeader>
            <CardTitle>SKU Images</CardTitle>
            <CardDescription className="text-neutral-500">
              Select a variant to manage its images.
            </CardDescription>
          </CardHeader>
        </Card>
      )}
    </div>
  );
}
