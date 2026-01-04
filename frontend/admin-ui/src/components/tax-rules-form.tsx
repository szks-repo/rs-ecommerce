"use client";

import { useEffect, useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { listTaxRules, upsertTaxRule } from "@/lib/store_settings";
import { getActiveAccessToken } from "@/lib/auth";
import { useToast } from "@/components/ui/toast";
import type { TaxRule } from "@/gen/ecommerce/v1/store_settings_pb";
import { useApiCall } from "@/lib/use-api-call";

export default function TaxRulesForm() {
  const [rules, setRules] = useState<TaxRule[]>([]);
  const [selectedRuleId, setSelectedRuleId] = useState("");
  const [name, setName] = useState("");
  const [ratePercent, setRatePercent] = useState("10");
  const [appliesTo, setAppliesTo] = useState("all");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const { push } = useToast();
  const { notifyError } = useApiCall();

  async function loadRules() {
    if (!getActiveAccessToken()) {
      return;
    }
    try {
      const data = await listTaxRules();
      setRules(data.rules ?? []);
    } catch (err) {
      notifyError(err, "Load failed", "Failed to load tax rules");
    }
  }

  useEffect(() => {
    void loadRules();
  }, []);

  useEffect(() => {
    const found = rules.find((rule) => rule.id === selectedRuleId);
    if (!found) {
      return;
    }
    setName(found.name);
    setRatePercent(String(found.rate * 100));
    setAppliesTo(found.appliesTo || "all");
  }, [rules, selectedRuleId]);

  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    setIsSubmitting(true);
    try {
      if (!getActiveAccessToken()) {
        throw new Error("access_token is missing. Please sign in first.");
      }
      const rateValue = Number(ratePercent);
      if (!Number.isFinite(rateValue)) {
        throw new Error("rate must be a number.");
      }
      await upsertTaxRule({
        id: selectedRuleId || undefined,
        name,
        rate: rateValue / 100,
        appliesTo,
      });
      push({
        variant: "success",
        title: "Tax rule saved",
        description: "Tax rule has been saved.",
      });
      setSelectedRuleId("");
      setName("");
      setRatePercent("10");
      setAppliesTo("all");
      await loadRules();
    } catch (err) {
      notifyError(err, "Save failed", "Unknown error");
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <Card className="border-neutral-200 bg-white text-neutral-900">
      <CardHeader>
        <CardTitle>Tax Rules</CardTitle>
        <CardDescription className="text-neutral-500">
          Define tax rates used for product pricing and shipping.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-2">
          <Label htmlFor="taxRuleSelect">Edit Existing Rule</Label>
          <Select value={selectedRuleId} onValueChange={setSelectedRuleId}>
            <SelectTrigger id="taxRuleSelect" className="bg-white">
              <SelectValue placeholder="Create new rule" />
            </SelectTrigger>
            <SelectContent>
              {rules.length === 0 ? (
                <SelectItem value="none" disabled>
                  No rules yet
                </SelectItem>
              ) : (
                rules.map((rule) => (
                  <SelectItem key={rule.id} value={rule.id}>
                    {rule.name} ({(rule.rate * 100).toFixed(1)}%)
                  </SelectItem>
                ))
              )}
            </SelectContent>
          </Select>
        </div>
        <form className="grid gap-4 md:grid-cols-3" onSubmit={handleSubmit}>
          <div className="space-y-2 md:col-span-2">
            <Label htmlFor="taxRuleName">Name</Label>
            <Input id="taxRuleName" value={name} onChange={(e) => setName(e.target.value)} required />
          </div>
          <div className="space-y-2">
            <Label htmlFor="taxRuleRate">Rate (%)</Label>
            <Input id="taxRuleRate" value={ratePercent} onChange={(e) => setRatePercent(e.target.value)} required />
          </div>
          <div className="space-y-2 md:col-span-2">
            <Label htmlFor="taxRuleApplies">Applies To</Label>
            <Select value={appliesTo} onValueChange={setAppliesTo}>
              <SelectTrigger id="taxRuleApplies" className="bg-white">
                <SelectValue placeholder="Select target" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">all</SelectItem>
                <SelectItem value="category">category</SelectItem>
                <SelectItem value="shipping">shipping</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="md:col-span-3">
            <Button type="submit" disabled={isSubmitting}>
              {isSubmitting ? "Saving..." : "Save Tax Rule"}
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  );
}
