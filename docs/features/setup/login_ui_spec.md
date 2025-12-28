# Backoffice Login UI Spec (Draft)

## Goal
- Provide admin/staff login for backoffice
- Obtain JWT from IdentityService
- Store access token in memory and attach to API requests

## UI Stack
- `shadcn/ui` components
- Tailwind CSS

## Pages

### 1) /login
#### Fields
- tenantId (text) or tenantName (optional; decide one)
- email
- password

#### Actions
- Submit -> `IdentityService/SignIn`
- On success: store access token in memory, optional refresh token in httpOnly cookie
- Redirect to `/admin`

#### Validation
- email format
- password non-empty
- tenantId required

#### Errors
- Show API error message
- Lock out after N failures (future)

### 2) /admin (shell)
- Layout with sidebar and topbar
- User menu: profile, logout

## Component Mapping (shadcn/ui)
- Form: `Form`, `FormField`, `FormItem`, `FormLabel`, `FormMessage`
- Inputs: `Input`, `Password` (Input type=password)
- Button: `Button`
- Card: `Card`
- Alert: `Alert`

## Auth Flow
1. User submits login
2. IdentityService returns JWT access token
3. UI stores access token in memory
4. API calls include `Authorization: Bearer <token>`
5. On 401, redirect to `/login`

## API Mapping
Request
```
POST /rpc/ecommerce.v1.IdentityService/SignIn
{
  "store": { "storeId": "..." },
  "email": "...",
  "password": "..."
}
```

Response
```
{
  "access_token": "...",
  "store_id": "...",
  "tenant_id": "...",
  "staff_id": "...",
  "role": "..."
}
```

## Open Questions
- tenant_id vs tenant_name input
- refresh token strategy
- remember me (future)
