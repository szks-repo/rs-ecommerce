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
  ListCustomerMetafieldDefinitionsRequestSchema,
  CreateCustomerMetafieldDefinitionRequestSchema,
  UpdateCustomerMetafieldDefinitionRequestSchema,
  CustomerMetafieldDefinitionInputSchema,
  ListCustomerMetafieldValuesRequestSchema,
  UpsertCustomerMetafieldValueRequestSchema,
} from "@/gen/ecommerce/v1/customer_pb";

const client = createServiceClient(CustomerService);

export async function listCustomers(params?: { query?: string; pageSize?: number; pageToken?: string }) {
  return client.listCustomers(
    create(ListCustomersRequestSchema, {
      query: params?.query || "",
      page: {
        pageSize: params?.pageSize ?? 50,
        pageToken: params?.pageToken ?? "",
      },
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
  countryCode?: string;
  identities?: { identityType: string; identityValue: string; verified?: boolean }[];
}) {
  const profile = create(CustomerProfileInputSchema, {
    name: params.name,
    email: params.email || "",
    phone: params.phone || "",
    status: params.status || "active",
    notes: params.notes || "",
    countryCode: params.countryCode || "JP",
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
  countryCode?: string;
  customerStatus?: string;
}) {
  const profile = create(CustomerProfileInputSchema, {
    name: params.name,
    email: params.email || "",
    phone: params.phone || "",
    status: params.status || "active",
    notes: params.notes || "",
    countryCode: params.countryCode || "JP",
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
  countryCode?: string;
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
    countryCode: params.countryCode || "JP",
  });
  return client.upsertCustomerAddress(
    create(UpsertCustomerAddressRequestSchema, {
      customerId: params.customerId,
      address,
    })
  );
}

export async function listCustomerMetafieldDefinitions() {
  return client.listCustomerMetafieldDefinitions(
    create(ListCustomerMetafieldDefinitionsRequestSchema, {})
  );
}

export async function createCustomerMetafieldDefinition(params: {
  namespace: string;
  key: string;
  name: string;
  description?: string;
  valueType: string;
  isList?: boolean;
  validationsJson?: string;
  visibilityJson?: string;
}) {
  const definition = create(CustomerMetafieldDefinitionInputSchema, {
    namespace: params.namespace,
    key: params.key,
    name: params.name,
    description: params.description || "",
    valueType: params.valueType,
    isList: params.isList ?? false,
    validationsJson: params.validationsJson || "",
    visibilityJson: params.visibilityJson || "",
  });
  return client.createCustomerMetafieldDefinition(
    create(CreateCustomerMetafieldDefinitionRequestSchema, {
      definition,
    })
  );
}

export async function updateCustomerMetafieldDefinition(params: {
  definitionId: string;
  namespace: string;
  key: string;
  name: string;
  description?: string;
  valueType: string;
  isList?: boolean;
  validationsJson?: string;
  visibilityJson?: string;
}) {
  const definition = create(CustomerMetafieldDefinitionInputSchema, {
    namespace: params.namespace,
    key: params.key,
    name: params.name,
    description: params.description || "",
    valueType: params.valueType,
    isList: params.isList ?? false,
    validationsJson: params.validationsJson || "",
    visibilityJson: params.visibilityJson || "",
  });
  return client.updateCustomerMetafieldDefinition(
    create(UpdateCustomerMetafieldDefinitionRequestSchema, {
      definitionId: params.definitionId,
      definition,
    })
  );
}

export async function listCustomerMetafieldValues(customerId: string) {
  return client.listCustomerMetafieldValues(
    create(ListCustomerMetafieldValuesRequestSchema, {
      customerId,
    })
  );
}

export async function upsertCustomerMetafieldValue(params: {
  customerId: string;
  definitionId: string;
  valueJson: string;
}) {
  return client.upsertCustomerMetafieldValue(
    create(UpsertCustomerMetafieldValueRequestSchema, {
      customerId: params.customerId,
      definitionId: params.definitionId,
      valueJson: params.valueJson,
    })
  );
}
