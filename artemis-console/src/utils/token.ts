/**
 * JWT Token Management Utility
 *
 * Provides functions for managing authentication tokens:
 * - Save/retrieve tokens from localStorage/sessionStorage
 * - Validate token expiration
 * - Parse JWT payload
 */

// ===== Constants =====

/**
 * Storage keys for authentication tokens
 */
const TOKEN_STORAGE_KEY = 'artemis_auth_token';
const TOKEN_REMEMBER_KEY = 'artemis_remember_me';

// ===== JWT Payload Interface =====

/**
 * Standard JWT payload structure
 */
export interface JwtPayload {
  sub: string; // Subject (user ID)
  username?: string;
  email?: string;
  roles?: string[];
  permissions?: string[];
  iat: number; // Issued at (timestamp in seconds)
  exp: number; // Expiration time (timestamp in seconds)
  [key: string]: any; // Allow other custom claims
}

// ===== Helper Functions =====

/**
 * Base64 URL decode
 * Converts base64url to standard base64, then decodes
 */
function base64UrlDecode(str: string): string {
  // Replace URL-safe characters
  let base64 = str.replace(/-/g, '+').replace(/_/g, '/');

  // Add padding if needed
  const padding = base64.length % 4;
  if (padding === 2) {
    base64 += '==';
  } else if (padding === 3) {
    base64 += '=';
  }

  // Decode base64
  try {
    return atob(base64);
  } catch (error) {
    throw new Error('Invalid base64 string');
  }
}

/**
 * Get storage implementation based on remember me setting
 */
function getStorage(): Storage {
  try {
    const remember = localStorage.getItem(TOKEN_REMEMBER_KEY);
    return remember === 'true' ? localStorage : sessionStorage;
  } catch {
    return sessionStorage;
  }
}

// ===== Public API =====

/**
 * Save authentication token
 *
 * @param token - JWT token string
 * @param remember - If true, store in localStorage; otherwise sessionStorage
 */
export function saveToken(token: string, remember: boolean = false): void {
  try {
    // Save remember preference
    localStorage.setItem(TOKEN_REMEMBER_KEY, remember.toString());

    // Save token to appropriate storage
    const storage = remember ? localStorage : sessionStorage;
    storage.setItem(TOKEN_STORAGE_KEY, token);

    // Clear token from the other storage
    const otherStorage = remember ? sessionStorage : localStorage;
    otherStorage.removeItem(TOKEN_STORAGE_KEY);
  } catch (error) {
    console.error('Failed to save token:', error);
    throw new Error('Failed to save authentication token');
  }
}

/**
 * Get authentication token
 *
 * @returns Token string if exists, null otherwise
 */
export function getToken(): string | null {
  try {
    // Try localStorage first (for remembered sessions)
    const localToken = localStorage.getItem(TOKEN_STORAGE_KEY);
    if (localToken) {
      return localToken;
    }

    // Fall back to sessionStorage
    const sessionToken = sessionStorage.getItem(TOKEN_STORAGE_KEY);
    return sessionToken;
  } catch (error) {
    console.error('Failed to get token:', error);
    return null;
  }
}

/**
 * Remove authentication token
 * Clears token from both localStorage and sessionStorage
 */
export function removeToken(): void {
  try {
    localStorage.removeItem(TOKEN_STORAGE_KEY);
    localStorage.removeItem(TOKEN_REMEMBER_KEY);
    sessionStorage.removeItem(TOKEN_STORAGE_KEY);
  } catch (error) {
    console.error('Failed to remove token:', error);
  }
}

/**
 * Parse JWT token payload
 *
 * @param token - JWT token string
 * @returns Decoded payload object
 * @throws Error if token is invalid
 */
export function getTokenPayload(token: string): JwtPayload {
  try {
    // JWT structure: header.payload.signature
    const parts = token.split('.');
    if (parts.length !== 3) {
      throw new Error('Invalid JWT structure');
    }

    // Decode payload (second part)
    const payloadBase64 = parts[1];
    const payloadJson = base64UrlDecode(payloadBase64);
    const payload = JSON.parse(payloadJson) as JwtPayload;

    return payload;
  } catch (error) {
    throw new Error(`Failed to parse JWT token: ${error instanceof Error ? error.message : 'Unknown error'}`);
  }
}

/**
 * Check if token is valid (not expired)
 *
 * @param token - JWT token string
 * @returns true if token is valid, false otherwise
 */
export function isTokenValid(token: string | null): boolean {
  if (!token) {
    return false;
  }

  try {
    const payload = getTokenPayload(token);

    // Check if token has expiration time
    if (!payload.exp) {
      // If no expiration, consider it invalid for security
      return false;
    }

    // Get current time in seconds (JWT uses seconds, not milliseconds)
    const currentTime = Math.floor(Date.now() / 1000);

    // Check if token is expired (with 60 second buffer)
    const isExpired = payload.exp < currentTime + 60;

    return !isExpired;
  } catch (error) {
    console.error('Token validation failed:', error);
    return false;
  }
}

/**
 * Get token expiration time
 *
 * @param token - JWT token string
 * @returns Expiration time as Date object, or null if invalid
 */
export function getTokenExpiration(token: string | null): Date | null {
  if (!token) {
    return null;
  }

  try {
    const payload = getTokenPayload(token);
    if (!payload.exp) {
      return null;
    }

    // Convert seconds to milliseconds
    return new Date(payload.exp * 1000);
  } catch {
    return null;
  }
}

/**
 * Get token issued time
 *
 * @param token - JWT token string
 * @returns Issued time as Date object, or null if invalid
 */
export function getTokenIssuedAt(token: string | null): Date | null {
  if (!token) {
    return null;
  }

  try {
    const payload = getTokenPayload(token);
    if (!payload.iat) {
      return null;
    }

    // Convert seconds to milliseconds
    return new Date(payload.iat * 1000);
  } catch {
    return null;
  }
}

/**
 * Check if token will expire soon (within specified minutes)
 *
 * @param token - JWT token string
 * @param minutes - Number of minutes threshold (default: 5)
 * @returns true if token will expire within the threshold
 */
export function willTokenExpireSoon(
  token: string | null,
  minutes: number = 5
): boolean {
  if (!token) {
    return true;
  }

  try {
    const payload = getTokenPayload(token);
    if (!payload.exp) {
      return true;
    }

    const currentTime = Math.floor(Date.now() / 1000);
    const threshold = minutes * 60;

    return payload.exp - currentTime < threshold;
  } catch {
    return true;
  }
}
