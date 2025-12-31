# Init WebUI Spec (Draft)

## Goal
- Guide initial store setup (similar to WordPress initial setup)
- Collect minimum required data for `SetupService.InitializeStore`
- After success, redirect to `/login`

## UI Stack
- `shadcn/ui` components
- Tailwind CSS

## Pages

### 1) /init (Wizard)
A step-based wizard. Each step validates before proceeding.

#### Step 0: Welcome
- Purpose: explain what will be created (tenant + store + owner staff)
- CTA: "Start setup"

#### Step 1: Store Basics
- storeName (required)

#### Step 2: Owner Account
- ownerEmail (required)
- ownerPassword (required)

#### Final Step: Review + Submit
- Show summary
- Submit to `SetupService.InitializeStore`

## Component Mapping (shadcn/ui)
- Form: `Form`, `FormField`, `FormItem`, `FormLabel`, `FormMessage`
- Inputs: `Input`, `Textarea`, `Select`, `Switch`, `Checkbox`
- Wizard: `Tabs` or custom stepper
- Summary: `Card`, `Table`
- CTA: `Button`

## Validation Rules (client side)
- Required fields must be filled
- Email format
- Password length (min 8 recommended)

## API Mapping
Request payload (JSON mapping):
- `storeName`
- `ownerEmail`
- `ownerPassword`

## Error Handling
- Show API error `message` on submit
- Step-level validation before continue

## Success State
- Show success toast
- Redirect to `/login`

## Future Enhancements
- Auto-fill prefectures for Japan
- Upload logo/favicon with storage integration
- Resume setup flow
