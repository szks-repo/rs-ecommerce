import { ErrorCode } from "@/gen/ecommerce/v1/common_pb";
import { parseErrorCode } from "@/lib/errors";

type UiError = {
  title: string;
  description: string;
};

export function formatConnectError(
  err: unknown,
  fallbackTitle = "Request failed",
  fallbackDescription = "Something went wrong"
): UiError {
  const code = parseErrorCode((err as { code?: string })?.code);
  const message =
    (err as { message?: string })?.message || fallbackDescription;

  switch (code) {
    case ErrorCode.ERROR_CODE_INVALID_ARGUMENT:
      return { title: "Invalid input", description: message };
    case ErrorCode.ERROR_CODE_NOT_FOUND:
      return { title: "Not found", description: message };
    case ErrorCode.ERROR_CODE_ALREADY_EXISTS:
      return { title: "Already exists", description: message };
    case ErrorCode.ERROR_CODE_PERMISSION_DENIED:
      return { title: "Permission denied", description: message };
    case ErrorCode.ERROR_CODE_UNAUTHENTICATED:
      return { title: "Sign in required", description: message };
    case ErrorCode.ERROR_CODE_FAILED_PRECONDITION:
      return { title: "Conflict", description: message };
    case ErrorCode.ERROR_CODE_INTERNAL:
      return { title: "Server error", description: message };
    case ErrorCode.ERROR_CODE_UNSUPPORTED_MEDIA_TYPE:
      return { title: "Unsupported media type", description: message };
    default:
      return { title: fallbackTitle, description: message };
  }
}
