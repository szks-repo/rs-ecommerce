# Setup API (draft)

## InitializeStore
Creates tenant + store + owner staff (settings configured later).

Request fields:
- store_name
- owner_email
- owner_password
- actor (optional)

Response fields:
- tenant_id
- store_id
- owner_staff_id
- vendor_id
