#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditAction {
    ProductCreate,
    ProductUpdate,
    VariantCreate,
    VariantUpdate,
    InventorySet,
    StoreSettingsUpdate,
    StoreSettingsInitialize,
    MallSettingsInitialize,
    MallSettingsUpdate,
    StoreLocationUpsert,
    StoreLocationDelete,
    ShippingZoneUpsert,
    ShippingZoneDelete,
    ShippingRateUpsert,
    ShippingRateDelete,
    TaxRuleUpsert,
    TaxRuleDelete,
    PromotionCreate,
    PromotionUpdate,
    AuctionCreate,
    AuctionBid,
    AuctionEnd,
    AuctionApprove,
    OrderUpdateStatus,
    ShipmentCreate,
    ShipmentUpdateStatus,
    IdentitySignIn,
    IdentitySignOut,
    IdentityStaffCreate,
    IdentityStaffUpdate,
    IdentityStaffInvite,
    IdentityRoleCreate,
    IdentityRoleAssign,
    IdentityRoleUpdate,
    IdentityRoleDelete,
    IdentityOwnerTransfer,
    CustomerCreate,
    CustomerUpdate,
    CustomerIdentityUpsert,
    CustomerAddressUpsert,
}

impl AuditAction {
    pub fn as_str(self) -> &'static str {
        match self {
            AuditAction::ProductCreate => "product.create",
            AuditAction::ProductUpdate => "product.update",
            AuditAction::VariantCreate => "variant.create",
            AuditAction::VariantUpdate => "variant.update",
            AuditAction::InventorySet => "inventory.set",
            AuditAction::StoreSettingsUpdate => "store_settings.update",
            AuditAction::StoreSettingsInitialize => "store_settings.initialize",
            AuditAction::MallSettingsInitialize => "mall_settings.initialize",
            AuditAction::MallSettingsUpdate => "mall_settings.update",
            AuditAction::StoreLocationUpsert => "store_location.upsert",
            AuditAction::StoreLocationDelete => "store_location.delete",
            AuditAction::ShippingZoneUpsert => "shipping_zone.upsert",
            AuditAction::ShippingZoneDelete => "shipping_zone.delete",
            AuditAction::ShippingRateUpsert => "shipping_rate.upsert",
            AuditAction::ShippingRateDelete => "shipping_rate.delete",
            AuditAction::TaxRuleUpsert => "tax_rule.upsert",
            AuditAction::TaxRuleDelete => "tax_rule.delete",
            AuditAction::PromotionCreate => "promotion.create",
            AuditAction::PromotionUpdate => "promotion.update",
            AuditAction::AuctionCreate => "auction.create",
            AuditAction::AuctionBid => "auction.bid",
            AuditAction::AuctionEnd => "auction.end",
            AuditAction::AuctionApprove => "auction.approve",
            AuditAction::OrderUpdateStatus => "order.update_status",
            AuditAction::ShipmentCreate => "shipment.create",
            AuditAction::ShipmentUpdateStatus => "shipment.update_status",
            AuditAction::IdentitySignIn => "identity.sign_in",
            AuditAction::IdentitySignOut => "identity.sign_out",
            AuditAction::IdentityStaffCreate => "identity.staff_create",
            AuditAction::IdentityStaffUpdate => "identity.staff_update",
            AuditAction::IdentityStaffInvite => "identity.staff_invite",
            AuditAction::IdentityRoleCreate => "identity.role_create",
            AuditAction::IdentityRoleAssign => "identity.role_assign",
            AuditAction::IdentityRoleUpdate => "identity.role_update",
            AuditAction::IdentityRoleDelete => "identity.role_delete",
            AuditAction::IdentityOwnerTransfer => "identity.owner_transfer",
            AuditAction::CustomerCreate => "customer.create",
            AuditAction::CustomerUpdate => "customer.update",
            AuditAction::CustomerIdentityUpsert => "customer.identity_upsert",
            AuditAction::CustomerAddressUpsert => "customer.address_upsert",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            AuditAction::ProductCreate => "Product created",
            AuditAction::ProductUpdate => "Product updated",
            AuditAction::VariantCreate => "Variant created",
            AuditAction::VariantUpdate => "Variant updated",
            AuditAction::InventorySet => "Inventory set",
            AuditAction::StoreSettingsUpdate => "Store settings updated",
            AuditAction::StoreSettingsInitialize => "Store settings initialized",
            AuditAction::MallSettingsInitialize => "Mall settings initialized",
            AuditAction::MallSettingsUpdate => "Mall settings updated",
            AuditAction::StoreLocationUpsert => "Store location saved",
            AuditAction::StoreLocationDelete => "Store location deleted",
            AuditAction::ShippingZoneUpsert => "Shipping zone saved",
            AuditAction::ShippingZoneDelete => "Shipping zone deleted",
            AuditAction::ShippingRateUpsert => "Shipping rate saved",
            AuditAction::ShippingRateDelete => "Shipping rate deleted",
            AuditAction::TaxRuleUpsert => "Tax rule saved",
            AuditAction::TaxRuleDelete => "Tax rule deleted",
            AuditAction::PromotionCreate => "Promotion created",
            AuditAction::PromotionUpdate => "Promotion updated",
            AuditAction::AuctionCreate => "Auction created",
            AuditAction::AuctionBid => "Auction bid placed",
            AuditAction::AuctionEnd => "Auction ended",
            AuditAction::AuctionApprove => "Auction approved",
            AuditAction::OrderUpdateStatus => "Order status updated",
            AuditAction::ShipmentCreate => "Shipment created",
            AuditAction::ShipmentUpdateStatus => "Shipment status updated",
            AuditAction::IdentitySignIn => "Signed in",
            AuditAction::IdentitySignOut => "Signed out",
            AuditAction::IdentityStaffCreate => "Staff created",
            AuditAction::IdentityStaffUpdate => "Staff updated",
            AuditAction::IdentityStaffInvite => "Staff invited",
            AuditAction::IdentityRoleCreate => "Role created",
            AuditAction::IdentityRoleAssign => "Role assigned",
            AuditAction::IdentityRoleUpdate => "Role updated",
            AuditAction::IdentityRoleDelete => "Role deleted",
            AuditAction::IdentityOwnerTransfer => "Owner transferred",
            AuditAction::CustomerCreate => "Customer created",
            AuditAction::CustomerUpdate => "Customer updated",
            AuditAction::CustomerIdentityUpsert => "Customer identity saved",
            AuditAction::CustomerAddressUpsert => "Customer address saved",
        }
    }
}

pub const ALL_AUDIT_ACTIONS: &[AuditAction] = &[
    AuditAction::ProductCreate,
    AuditAction::ProductUpdate,
    AuditAction::VariantCreate,
    AuditAction::VariantUpdate,
    AuditAction::InventorySet,
    AuditAction::StoreSettingsUpdate,
    AuditAction::StoreSettingsInitialize,
    AuditAction::MallSettingsInitialize,
    AuditAction::MallSettingsUpdate,
    AuditAction::StoreLocationUpsert,
    AuditAction::StoreLocationDelete,
    AuditAction::ShippingZoneUpsert,
    AuditAction::ShippingZoneDelete,
    AuditAction::ShippingRateUpsert,
    AuditAction::ShippingRateDelete,
    AuditAction::TaxRuleUpsert,
    AuditAction::TaxRuleDelete,
    AuditAction::PromotionCreate,
    AuditAction::PromotionUpdate,
    AuditAction::AuctionCreate,
    AuditAction::AuctionBid,
    AuditAction::AuctionEnd,
    AuditAction::AuctionApprove,
    AuditAction::OrderUpdateStatus,
    AuditAction::ShipmentCreate,
    AuditAction::ShipmentUpdateStatus,
    AuditAction::IdentitySignIn,
    AuditAction::IdentitySignOut,
    AuditAction::IdentityStaffCreate,
    AuditAction::IdentityStaffUpdate,
    AuditAction::IdentityStaffInvite,
    AuditAction::IdentityRoleCreate,
    AuditAction::IdentityRoleAssign,
    AuditAction::IdentityRoleUpdate,
    AuditAction::IdentityRoleDelete,
    AuditAction::IdentityOwnerTransfer,
    AuditAction::CustomerCreate,
    AuditAction::CustomerUpdate,
    AuditAction::CustomerIdentityUpsert,
    AuditAction::CustomerAddressUpsert,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductAuditAction {
    Create,
    Update,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariantAuditAction {
    Create,
    Update,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventoryAuditAction {
    Set,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreSettingsAuditAction {
    Update,
    Initialize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MallSettingsAuditAction {
    Initialize,
    Update,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreLocationAuditAction {
    Upsert,
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShippingZoneAuditAction {
    Upsert,
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShippingRateAuditAction {
    Upsert,
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaxRuleAuditAction {
    Upsert,
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromotionAuditAction {
    Create,
    Update,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuctionAuditAction {
    Create,
    Bid,
    End,
    Approve,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderAuditAction {
    UpdateStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShipmentAuditAction {
    Create,
    UpdateStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityAuditAction {
    SignIn,
    SignOut,
    StaffCreate,
    StaffUpdate,
    StaffInvite,
    RoleCreate,
    RoleAssign,
    RoleUpdate,
    RoleDelete,
    OwnerTransfer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CustomerAuditAction {
    Create,
    Update,
    IdentityUpsert,
    AddressUpsert,
}

impl From<ProductAuditAction> for AuditAction {
    fn from(action: ProductAuditAction) -> Self {
        match action {
            ProductAuditAction::Create => AuditAction::ProductCreate,
            ProductAuditAction::Update => AuditAction::ProductUpdate,
        }
    }
}

impl From<VariantAuditAction> for AuditAction {
    fn from(action: VariantAuditAction) -> Self {
        match action {
            VariantAuditAction::Create => AuditAction::VariantCreate,
            VariantAuditAction::Update => AuditAction::VariantUpdate,
        }
    }
}

impl From<InventoryAuditAction> for AuditAction {
    fn from(action: InventoryAuditAction) -> Self {
        match action {
            InventoryAuditAction::Set => AuditAction::InventorySet,
        }
    }
}

impl From<StoreSettingsAuditAction> for AuditAction {
    fn from(action: StoreSettingsAuditAction) -> Self {
        match action {
            StoreSettingsAuditAction::Update => AuditAction::StoreSettingsUpdate,
            StoreSettingsAuditAction::Initialize => AuditAction::StoreSettingsInitialize,
        }
    }
}

impl From<MallSettingsAuditAction> for AuditAction {
    fn from(action: MallSettingsAuditAction) -> Self {
        match action {
            MallSettingsAuditAction::Initialize => AuditAction::MallSettingsInitialize,
            MallSettingsAuditAction::Update => AuditAction::MallSettingsUpdate,
        }
    }
}

impl From<StoreLocationAuditAction> for AuditAction {
    fn from(action: StoreLocationAuditAction) -> Self {
        match action {
            StoreLocationAuditAction::Upsert => AuditAction::StoreLocationUpsert,
            StoreLocationAuditAction::Delete => AuditAction::StoreLocationDelete,
        }
    }
}

impl From<ShippingZoneAuditAction> for AuditAction {
    fn from(action: ShippingZoneAuditAction) -> Self {
        match action {
            ShippingZoneAuditAction::Upsert => AuditAction::ShippingZoneUpsert,
            ShippingZoneAuditAction::Delete => AuditAction::ShippingZoneDelete,
        }
    }
}

impl From<ShippingRateAuditAction> for AuditAction {
    fn from(action: ShippingRateAuditAction) -> Self {
        match action {
            ShippingRateAuditAction::Upsert => AuditAction::ShippingRateUpsert,
            ShippingRateAuditAction::Delete => AuditAction::ShippingRateDelete,
        }
    }
}

impl From<TaxRuleAuditAction> for AuditAction {
    fn from(action: TaxRuleAuditAction) -> Self {
        match action {
            TaxRuleAuditAction::Upsert => AuditAction::TaxRuleUpsert,
            TaxRuleAuditAction::Delete => AuditAction::TaxRuleDelete,
        }
    }
}

impl From<PromotionAuditAction> for AuditAction {
    fn from(action: PromotionAuditAction) -> Self {
        match action {
            PromotionAuditAction::Create => AuditAction::PromotionCreate,
            PromotionAuditAction::Update => AuditAction::PromotionUpdate,
        }
    }
}

impl From<AuctionAuditAction> for AuditAction {
    fn from(action: AuctionAuditAction) -> Self {
        match action {
            AuctionAuditAction::Create => AuditAction::AuctionCreate,
            AuctionAuditAction::Bid => AuditAction::AuctionBid,
            AuctionAuditAction::End => AuditAction::AuctionEnd,
            AuctionAuditAction::Approve => AuditAction::AuctionApprove,
        }
    }
}

impl From<OrderAuditAction> for AuditAction {
    fn from(action: OrderAuditAction) -> Self {
        match action {
            OrderAuditAction::UpdateStatus => AuditAction::OrderUpdateStatus,
        }
    }
}

impl From<ShipmentAuditAction> for AuditAction {
    fn from(action: ShipmentAuditAction) -> Self {
        match action {
            ShipmentAuditAction::Create => AuditAction::ShipmentCreate,
            ShipmentAuditAction::UpdateStatus => AuditAction::ShipmentUpdateStatus,
        }
    }
}

impl From<IdentityAuditAction> for AuditAction {
    fn from(action: IdentityAuditAction) -> Self {
        match action {
            IdentityAuditAction::SignIn => AuditAction::IdentitySignIn,
            IdentityAuditAction::SignOut => AuditAction::IdentitySignOut,
            IdentityAuditAction::StaffCreate => AuditAction::IdentityStaffCreate,
            IdentityAuditAction::StaffUpdate => AuditAction::IdentityStaffUpdate,
            IdentityAuditAction::StaffInvite => AuditAction::IdentityStaffInvite,
            IdentityAuditAction::RoleCreate => AuditAction::IdentityRoleCreate,
            IdentityAuditAction::RoleAssign => AuditAction::IdentityRoleAssign,
            IdentityAuditAction::RoleUpdate => AuditAction::IdentityRoleUpdate,
            IdentityAuditAction::RoleDelete => AuditAction::IdentityRoleDelete,
            IdentityAuditAction::OwnerTransfer => AuditAction::IdentityOwnerTransfer,
        }
    }
}

impl From<CustomerAuditAction> for AuditAction {
    fn from(action: CustomerAuditAction) -> Self {
        match action {
            CustomerAuditAction::Create => AuditAction::CustomerCreate,
            CustomerAuditAction::Update => AuditAction::CustomerUpdate,
            CustomerAuditAction::IdentityUpsert => AuditAction::CustomerIdentityUpsert,
            CustomerAuditAction::AddressUpsert => AuditAction::CustomerAddressUpsert,
        }
    }
}
