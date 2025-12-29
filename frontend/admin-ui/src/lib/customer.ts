import { create } from "@bufbuild/protobuf";
import { createServiceClient } from "@/lib/connect";
import {
  CustomerService,
  ListCustomersRequestSchema,
  GetCustomerRequestSchema,
  CreateCustomerRequestSchema,
  UpdateCustomerRequestSchema,
  CustomerProfileInputSchema,
  CustomerIdentityInputSchema,
  CustomerIdentityUpsertSchema,
  CustomerAddressInputSchema,
  UpsertCustomerIdentityRequestSchema,
  UpsertCustomerAddressRequestSchema,
} from "@/gen/ecommerce/v1/customer_pb";

const client = createServiceClient(CustomerService);

export async function listCustomers(params?: { query?: string }) {
  return client.listCustomers(
    create(ListCustomersRequestSchema, {
      query: params?.query || "",
    })
  );
}

export async function getCustomer(customerId: string) {
  return client.getCustomer(
    create(GetCustomerRequestSchema, {
      customerId,
    })
  );
}

export async function createCustomer(params: {
  name: string;
  email?: string;
  phone?: string;
  status?: string;
  notes?: string;
  identities?: { identityType: string; identityValue: string; verified?: boolean }[];
}) {
  const profile = create(CustomerProfileInputSchema, {
    name: params.name,
    email: params.email || "",
    phone: params.phone || "",
    status: params.status || "active",
    notes: params.notes || "",
  });
  const identities = (params.identities || []).map((identity) =>
    create(CustomerIdentityInputSchema, {
      identityType: identity.identityType,
      identityValue: identity.identityValue,
      verified: identity.verified ?? false,
    })
  );
  return client.createCustomer(
    create(CreateCustomerRequestSchema, {
      profile,
      identities,
    })
  );
}

export async function updateCustomer(params: {
  customerId: string;
  name: string;
  email?: string;
  phone?: string;
  status?: string;
  notes?: string;
  customerStatus?: string;
}) {
  const profile = create(CustomerProfileInputSchema, {
    name: params.name,
    email: params.email || "",
    phone: params.phone || "",
    status: params.status || "active",
    notes: params.notes || "",
  });
  return client.updateCustomer(
    create(UpdateCustomerRequestSchema, {
      customerId: params.customerId,
      profile,
      customerStatus: params.customerStatus || "",
    })
  );
}

export async function upsertCustomerIdentity(params: {
  customerId: string;
  id?: string;
  identityType: string;
  identityValue: string;
  verified?: boolean;
  source?: string;
}) {
  const identity = create(CustomerIdentityUpsertSchema, {
    id: params.id || "",
    identityType: params.identityType,
    identityValue: params.identityValue,
    verified: params.verified ?? false,
    source: params.source || "",
  });
  return client.upsertCustomerIdentity(
    create(UpsertCustomerIdentityRequestSchema, {
      customerId: params.customerId,
      identity,
    })
  );
}

export async function upsertCustomerAddress(params: {
  customerId: string;
  id?: string;
  type: string;
  name: string;
  postalCode: string;
  prefecture: string;
  city: string;
  line1: string;
  line2?: string;
  phone?: string;
}) {
  const address = create(CustomerAddressInputSchema, {
    id: params.id || "",
    type: params.type,
    name: params.name,
    postalCode: params.postalCode,
    prefecture: params.prefecture,
    city: params.city,
    line1: params.line1,
    line2: params.line2 || "",
    phone: params.phone || "",
  });
  return client.upsertCustomerAddress(
    create(UpsertCustomerAddressRequestSchema, {
      customerId: params.customerId,
      address,
    })
  );
}
