"use client";

import { useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useToast } from "@/components/ui/toast";
import { createCustomer } from "@/lib/customer";
import { useApiCall } from "@/lib/use-api-call";

const COUNTRY_OPTIONS = [
  { code: "JP", label: "Japan (JP)" },
  { code: "US", label: "United States (US)" },
  { code: "GB", label: "United Kingdom (GB)" },
];

export default function CustomerCreateForm() {
  const [name, setName] = useState("");
  const [email, setEmail] = useState("");
  const [phone, setPhone] = useState("");
  const [notes, setNotes] = useState("");
  const [status, setStatus] = useState("active");
  const [countryCode, setCountryCode] = useState("JP");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const { push } = useToast();
  const { notifyError } = useApiCall();

  async function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (isSubmitting) {
      return;
    }
    setIsSubmitting(true);
    try {
      const resp = await createCustomer({
        name,
        email: email || undefined,
        phone: phone || undefined,
        status,
        notes: notes || undefined,
        countryCode,
      });
      push({
        variant: "success",
        title: "Customer created",
        description: resp.matchedExisting
          ? "Matched an existing customer and linked the profile."
          : "New customer record created.",
      });
      setName("");
      setEmail("");
      setPhone("");
      setNotes("");
      setStatus("active");
      setCountryCode("JP");
    } catch (err) {
      notifyError(err, "Create failed", "Failed to create customer");
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <Card className="border-neutral-200 bg-white text-neutral-900">
      <CardHeader>
        <CardTitle>Create Customer</CardTitle>
        <CardDescription className="text-neutral-500">
          Register a customer and link identity information.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form className="space-y-4" onSubmit={handleSubmit}>
          <div className="space-y-2">
            <Label htmlFor="customerName">Name</Label>
            <Input
              id="customerName"
              value={name}
              onChange={(event) => setName(event.target.value)}
              placeholder="Customer name"
            />
          </div>
          <div className="grid gap-4 md:grid-cols-2">
            <div className="space-y-2">
              <Label htmlFor="customerEmail">Email</Label>
              <Input
                id="customerEmail"
                value={email}
                onChange={(event) => setEmail(event.target.value)}
                placeholder="example@domain.com"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="customerPhone">Phone</Label>
              <Input
                id="customerPhone"
                value={phone}
                onChange={(event) => setPhone(event.target.value)}
                placeholder="09000000000"
              />
            </div>
          </div>
          <div className="space-y-2">
            <Label>Status</Label>
            <Select value={status} onValueChange={setStatus}>
              <SelectTrigger className="bg-white">
                <SelectValue placeholder="Select status" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="active">active</SelectItem>
                <SelectItem value="inactive">inactive</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-2">
            <Label>Country</Label>
            <Select value={countryCode} onValueChange={setCountryCode}>
              <SelectTrigger className="bg-white">
                <SelectValue placeholder="Select country" />
              </SelectTrigger>
              <SelectContent>
                {COUNTRY_OPTIONS.map((country) => (
                  <SelectItem key={country.code} value={country.code}>
                    {country.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-2">
            <Label htmlFor="customerNotes">Notes</Label>
            <Textarea
              id="customerNotes"
              value={notes}
              onChange={(event) => setNotes(event.target.value)}
              placeholder="Internal notes"
            />
          </div>
          <Button type="submit" disabled={isSubmitting}>
            {isSubmitting ? "Creating..." : "Create customer"}
          </Button>
        </form>
      </CardContent>
    </Card>
  );
}
