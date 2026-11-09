import { KNOWN_ERROR_CODES } from "../constants/ERROR_CODES";

export type AppErrorCode = (typeof KNOWN_ERROR_CODES)[number];
