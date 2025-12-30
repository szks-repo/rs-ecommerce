import { ErrorCode } from "@/gen/ecommerce/v1/common_pb";

export type ErrorCodeString =
  | "invalid_argument"
  | "not_found"
  | "already_exists"
  | "permission_denied"
  | "unauthenticated"
  | "failed_precondition"
  | "internal"
  | "unsupported_media_type";

export function parseErrorCode(code?: string): ErrorCode | undefined {
  switch (code) {
    case "invalid_argument":
      return ErrorCode.ERROR_CODE_INVALID_ARGUMENT;
    case "not_found":
      return ErrorCode.ERROR_CODE_NOT_FOUND;
    case "already_exists":
      return ErrorCode.ERROR_CODE_ALREADY_EXISTS;
    case "permission_denied":
      return ErrorCode.ERROR_CODE_PERMISSION_DENIED;
    case "unauthenticated":
      return ErrorCode.ERROR_CODE_UNAUTHENTICATED;
    case "failed_precondition":
      return ErrorCode.ERROR_CODE_FAILED_PRECONDITION;
    case "internal":
      return ErrorCode.ERROR_CODE_INTERNAL;
    case "unsupported_media_type":
      return ErrorCode.ERROR_CODE_UNSUPPORTED_MEDIA_TYPE;
    default:
      return undefined;
  }
}
