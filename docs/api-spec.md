# Photonic API Specification

RESTful API documentation for Photonic photo management system.

## Table of Contents

- [Overview](#overview)
- [Authentication](#authentication)
- [Error Handling](#error-handling)
- [User Endpoints](#user-endpoints)
- [Media Upload Endpoints](#media-upload-endpoints)
- [Media Retrieval Endpoints](#media-retrieval-endpoints)
- [Media Management Endpoints](#media-management-endpoints)
- [Album Endpoints (Future)](#album-endpoints-future)
- [System Endpoints](#system-endpoints)

---

## Overview

### Base URL

```
Production:  https://api.photonic.example.com
Development: http://localhost:8080
```

### API Versioning

All endpoints are prefixed with `/api/v1/`

```
https://api.photonic.example.com/api/v1/media
```

### Content Types

**Request:**

- `application/json` - JSON request bodies
- `multipart/form-data` - File uploads

**Response:**

- `application/json` - JSON responses
- `image/*`, `video/*` - File downloads

### Rate Limiting

```
- 1000 requests per hour per user
- 10 uploads per minute per user
- Headers: X-RateLimit-Limit, X-RateLimit-Remaining, X-RateLimit-Reset
```

---

## Authentication

### OAuth2 / JWT

All endpoints (except `/health`) require authentication via JWT Bearer token.

**Authorization Header:**

```http
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

### JWT Claims

Expected JWT payload structure:

```json
{
  "sub": "550e8400-e29b-41d4-a716-446655440000",
  "username": "john_doe",
  "email": "john@example.com",
  "quota": 10737418240,
  "iss": "https://idp.example.com",
  "aud": "photonic-api",
  "exp": 1734361200,
  "iat": 1734357600
}
```

**Required Claims:**

- `sub` - User ID (UUID format)
- `username` - Username (string)
- `quota` - Storage quota in bytes (integer)

**Optional Claims:**

- `email` - User email (string)

### Authentication Errors

```json
{
  "error": "unauthorized",
  "message": "Missing or invalid authentication token"
}
```

**Status Codes:**

- `401 Unauthorized` - Missing or invalid token
- `403 Forbidden` - Valid token but insufficient permissions

---

## Error Handling

### Error Response Format

All error responses follow this structure:

```json
{
  "error": "error_code",
  "message": "Human-readable error message",
  "details": {
    // Optional: Additional error context
  }
}
```

### Common Error Codes

| Status Code | Error Code            | Description                          |
|-------------|-----------------------|--------------------------------------|
| 400         | `bad_request`         | Invalid request format or parameters |
| 401         | `unauthorized`        | Missing or invalid authentication    |
| 403         | `forbidden`           | Insufficient permissions             |
| 404         | `not_found`           | Resource not found                   |
| 409         | `conflict`            | Resource conflict (e.g., duplicate)  |
| 413         | `quota_exceeded`      | Storage quota exceeded               |
| 422         | `validation_error`    | Request validation failed            |
| 429         | `rate_limit_exceeded` | Too many requests                    |
| 500         | `internal_error`      | Server error                         |
| 503         | `service_unavailable` | Temporary service unavailability     |

### Validation Error Example

```json
{
  "error": "validation_error",
  "message": "Request validation failed",
  "details": {
    "fields": {
      "username": "Username must be 3-50 characters",
      "email": "Invalid email format"
    }
  }
}
```

---

## User Endpoints

### Get Current User Profile

```http
GET /api/v1/users/me
```

**Description:** Retrieve current user's profile and quota information.

**Authentication:** Required

**Response:**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "username": "john_doe",
  "email": "john@example.com",
  "quota": {
    "total_bytes": 10737418240,
    "used_bytes": 8388608,
    "reserved_bytes": 1048576,
    "available_bytes": 10727980656
  },
  "created_at": "2024-01-15T10:00:00Z",
  "updated_at": "2024-12-16T09:00:00Z"
}
```

**Status Codes:**

- `200 OK` - Success
- `401 Unauthorized` - Invalid token

---

### Get Quota Status

```http
GET /api/v1/users/me/quota
```

**Description:** Detailed quota information.

**Authentication:** Required

**Response:**

```json
{
  "quota_bytes": 10737418240,
  "used_bytes": 8388608,
  "reserved_bytes": 1048576,
  "available_bytes": 10727980656,
  "usage_percentage": 0.08,
  "media_count": 145,
  "largest_medium": {
    "id": "medium-uuid",
    "filename": "IMG_9876.cr2",
    "size_bytes": 52428800
  }
}
```

**Status Codes:**

- `200 OK` - Success
- `401 Unauthorized` - Invalid token

---

## Media Upload Endpoints

### Upload Medium

```http
POST /api/v1/media
```

**Description:** Upload a new photo or video.

**Authentication:** Required

**Content-Type:** `multipart/form-data`

**Form Fields:**

- `file` (required) - The media file
- `tags` (optional) - Comma-separated tags
- `album_id` (optional) - Album UUID
- `priority` (optional) - Processing priority (1-10)

**Request Example:**

```http
POST /api/v1/media HTTP/1.1
Host: api.photonic.example.com
Authorization: Bearer {token}
Content-Type: multipart/form-data; boundary=----Boundary

------Boundary
Content-Disposition: form-data; name="file"; filename="IMG_1234.jpg"
Content-Type: image/jpeg

{binary data}
------Boundary
Content-Disposition: form-data; name="tags"

vacation,landscape,summer
------Boundary
Content-Disposition: form-data; name="album_id"

770e8400-e29b-41d4-a716-446655440000
------Boundary--
```

**Response:**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "task",
  "message": "Upload successful. Processing started.",
  "created_at": "2024-12-16T10:35:00Z"
}
```

**Status Codes:**

- `201 Created` - Upload successful
- `400 Bad Request` - Invalid file or parameters
- `413 Payload Too Large` - Quota exceeded
- `422 Unprocessable Entity` - File type not supported
- `500 Internal Server Error` - Upload failed

**Error Example (Quota Exceeded):**

```json
{
  "error": "quota_exceeded",
  "message": "Insufficient quota. Required: 52428800 bytes, Available: 1048576 bytes",
  "details": {
    "required_bytes": 52428800,
    "available_bytes": 1048576,
    "quota_bytes": 10737418240,
    "used_bytes": 10736369664
  }
}
```

---

## Media Retrieval Endpoints

### Get Medium Metadata

```http
GET /api/v1/media/{medium_id}
```

**Description:** Retrieve detailed metadata for a specific medium.

**Authentication:** Required

**Path Parameters:**

- `medium_id` (UUID) - Medium identifier

**Response:**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "user_id": "660e8400-e29b-41d4-a716-446655440000",
  "type": "Photo",
  "original_filename": "IMG_1234.jpg",
  "state": "Ready",
  "metadata": {
    "taken_at": "2024-12-15T14:22:00Z",
    "camera_make": "Canon",
    "camera_model": "EOS R5",
    "lens_model": "RF 24-70mm F2.8 L IS USM",
    "iso": 400,
    "aperture": 2.8,
    "shutter_speed": "1/250",
    "focal_length": 50.0,
    "gps_coordinates": {
      "latitude": 52.520008,
      "longitude": 13.404954
    }
  },
  "items": [
    {
      "id": "item-uuid-1",
      "type": "Original",
      "mime_type": "image/jpeg",
      "file_size": 8388608,
      "dimensions": {"width": 8192, "height": 5464},
      "state": "Ready",
      "url": "/api/v1/media/550e8400-e29b-41d4-a716-446655440000/items/item-uuid-1"
    },
    {
      "id": "item-uuid-2",
      "type": "Thumbnail",
      "mime_type": "image/jpeg",
      "file_size": 45000,
      "dimensions": {"width": 200, "height": 200},
      "state": "Ready",
      "url": "/api/v1/media/550e8400-e29b-41d4-a716-446655440000/items/item-uuid-2"
    },
    {
      "id": "item-uuid-3",
      "type": "Preview",
      "mime_type": "image/jpeg",
      "file_size": 524288,
      "dimensions": {"width": 1024, "height": 683},
      "state": "Ready",
      "url": "/api/v1/media/550e8400-e29b-41d4-a716-446655440000/items/item-uuid-3"
    }
  ],
  "tags": ["vacation", "landscape", "summer"],
  "album_id": "770e8400-e29b-41d4-a716-446655440000",
  "created_at": "2024-12-16T10:35:00Z",
  "updated_at": "2024-12-16T10:36:15Z"
}
```

**Status Codes:**

- `200 OK` - Success
- `404 Not Found` - Medium not found or no access
- `403 Forbidden` - No permission to access

---

### Download Medium Item (Variant)

```http
GET /api/v1/media/{medium_id}/items/{item_id}
```

**Description:** Download a specific variant of the medium (original, thumbnail, preview, etc.).

**Authentication:** Required

**Path Parameters:**

- `medium_id` (UUID) - Medium identifier
- `item_id` (UUID) - Item identifier

**Query Parameters:**

- `download` (boolean, optional) - If true, sets Content-Disposition to attachment

**Response Headers:**

```http
HTTP/1.1 200 OK
Content-Type: image/jpeg
Content-Length: 8388608
Content-Disposition: inline
Cache-Control: public, max-age=31536000
ETag: "abc123def456"

{binary data}
```

**Status Codes:**

- `200 OK` - File retrieved
- `202 Accepted` - Item still processing (retry later)
- `404 Not Found` - Medium or item not found
- `403 Forbidden` - No permission
- `500 Internal Server Error` - Variant generation failed

**Processing Response (202):**

```http
HTTP/1.1 202 Accepted
Retry-After: 30
Content-Type: application/json

{
  "status": "processing",
  "message": "Item is being processed. Please retry after 30 seconds."
}
```

---

### List User Media

```http
GET /api/v1/media
```

**Description:** List user's media with filtering and pagination.

**Authentication:** Required

**Query Parameters:**

- `page` (integer, default: 1) - Page number
- `limit` (integer, default: 20, max: 100) - Items per page
- `type` (string, optional) - Filter by type: `Photo`, `Video`, `LivePhoto`, `Other`
- `album_id` (UUID, optional) - Filter by album
- `tags` (string, optional) - Comma-separated tags (OR logic)
- `date_from` (ISO 8601, optional) - Filter by taken_at >= date
- `date_to` (ISO 8601, optional) - Filter by taken_at <= date
- `sort` (string, default: `created_at:desc`) - Sort field and order
    - Valid values: `created_at:asc`, `created_at:desc`, `taken_at:asc`, `taken_at:desc`
- `state` (string, optional) - Filter by state: `Uploading`, `Processing`, `Ready`, `Failed`

**Request Example:**

```http
GET /api/v1/media?page=1&limit=20&type=Photo&tags=vacation,landscape&sort=taken_at:desc HTTP/1.1
Host: api.photonic.example.com
Authorization: Bearer {token}
```

**Response:**

```json
{
  "total": 156,
  "page": 1,
  "limit": 20,
  "pages": 8,
  "items": [
    {
      "id": "uuid-1",
      "type": "Photo",
      "filename": "IMG_1234.jpg",
      "thumbnail_url": "/api/v1/media/uuid-1/items/thumb-uuid",
      "taken_at": "2024-12-16T10:30:00Z",
      "state": "Ready",
      "tags": ["vacation", "landscape"],
      "dimensions": {"width": 8192, "height": 5464},
      "file_size": 8388608
    },
    {
      "id": "uuid-2",
      "type": "Photo",
      "filename": "IMG_1235.jpg",
      "thumbnail_url": "/api/v1/media/uuid-2/items/thumb-uuid",
      "taken_at": "2024-12-16T10:28:00Z",
      "state": "Ready",
      "tags": ["vacation"],
      "dimensions": {"width": 8192, "height": 5464},
      "file_size": 8388608
    }
  ],
  "links": {
    "self": "/api/v1/media?page=1&limit=20&type=Photo&tags=vacation,landscape",
    "first": "/api/v1/media?page=1&limit=20&type=Photo&tags=vacation,landscape",
    "prev": null,
    "next": "/api/v1/media?page=2&limit=20&type=Photo&tags=vacation,landscape",
    "last": "/api/v1/media?page=8&limit=20&type=Photo&tags=vacation,landscape"
  }
}
```

**Status Codes:**

- `200 OK` - Success
- `400 Bad Request` - Invalid query parameters

---

### Search Media

```http
GET /api/v1/media/search
```

**Description:** Full-text search across media metadata.

**Authentication:** Required

**Query Parameters:**

- `q` (string, required) - Search query
- All parameters from List Media endpoint

**Search Fields:**

- Tags (exact and partial match)
- Original filename
- Camera make and model
- Metadata fields (lens, location, etc.)

**Request Example:**

```http
GET /api/v1/media/search?q=canon&page=1&limit=20 HTTP/1.1
Host: api.photonic.example.com
Authorization: Bearer {token}
```

**Response:**

```json
{
  "query": "canon",
  "total": 45,
  "page": 1,
  "limit": 20,
  "pages": 3,
  "items": [
    {
      "id": "uuid-1",
      "type": "Photo",
      "filename": "IMG_1234.jpg",
      "thumbnail_url": "/api/v1/media/uuid-1/items/thumb-uuid",
      "taken_at": "2024-12-15T14:22:00Z",
      "metadata": {
        "camera_make": "Canon",
        "camera_model": "EOS R5"
      },
      "relevance": 0.95,
      "match_reason": "camera_make"
    }
  ],
  "links": {
    "self": "/api/v1/media/search?q=canon&page=1&limit=20",
    "next": "/api/v1/media/search?q=canon&page=2&limit=20"
  }
}
```

**Status Codes:**

- `200 OK` - Success
- `400 Bad Request` - Missing or invalid query

---

## Media Management Endpoints

### Delete Medium

```http
DELETE /api/v1/media/{medium_id}
```

**Description:** Delete medium and all its variants, freeing up quota.

**Authentication:** Required

**Path Parameters:**

- `medium_id` (UUID) - Medium identifier

**Response:**

```http
HTTP/1.1 204 No Content
```

**Status Codes:**

- `204 No Content` - Successfully deleted
- `404 Not Found` - Medium not found
- `403 Forbidden` - No permission to delete

---

### Add Tags to Medium

```http
POST /api/v1/media/{medium_id}/tags
```

**Description:** Add tags to a medium.

**Authentication:** Required

**Path Parameters:**

- `medium_id` (UUID) - Medium identifier

**Request Body:**

```json
{
  "tags": ["vacation", "landscape", "summer"]
}
```

**Response:**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "tags": ["vacation", "landscape", "summer", "mountains"],
  "updated_at": "2024-12-16T11:00:00Z"
}
```

**Status Codes:**

- `200 OK` - Tags added
- `400 Bad Request` - Invalid tags
- `404 Not Found` - Medium not found
- `403 Forbidden` - No permission

**Validation Rules:**

- Tag length: 1-50 characters
- Allowed characters: lowercase alphanumeric, hyphen, underscore
- Max 50 tags per medium
- Automatically converted to lowercase
- Duplicates ignored

---

### Remove Tags from Medium

```http
DELETE /api/v1/media/{medium_id}/tags
```

**Description:** Remove tags from a medium.

**Authentication:** Required

**Path Parameters:**

- `medium_id` (UUID) - Medium identifier

**Request Body:**

```json
{
  "tags": ["summer"]
}
```

**Response:**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "tags": ["vacation", "landscape", "mountains"],
  "updated_at": "2024-12-16T11:05:00Z"
}
```

**Status Codes:**

- `200 OK` - Tags removed
- `404 Not Found` - Medium not found
- `403 Forbidden` - No permission

---

### Assign Medium to Album

```http
PUT /api/v1/media/{medium_id}/album
```

**Description:** Assign medium to an album.

**Authentication:** Required

**Path Parameters:**

- `medium_id` (UUID) - Medium identifier

**Request Body:**

```json
{
  "album_id": "770e8400-e29b-41d4-a716-446655440000"
}
```

**Response:**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "album_id": "770e8400-e29b-41d4-a716-446655440000",
  "updated_at": "2024-12-16T11:10:00Z"
}
```

**Status Codes:**

- `200 OK` - Album assigned
- `404 Not Found` - Medium or album not found
- `403 Forbidden` - No permission

---

### Remove Medium from Album

```http
DELETE /api/v1/media/{medium_id}/album
```

**Description:** Remove medium from album.

**Authentication:** Required

**Path Parameters:**

- `medium_id` (UUID) - Medium identifier

**Response:**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "album_id": null,
  "updated_at": "2024-12-16T11:15:00Z"
}
```

**Status Codes:**

- `200 OK` - Removed from album
- `404 Not Found` - Medium not found
- `403 Forbidden` - No permission

---

## Album Endpoints (Future)

### Create Album

```http
POST /api/v1/albums
```

**Request Body:**

```json
{
  "title": "Vacation 2024",
  "description": "Summer trip to the mountains",
  "parent_id": null
}
```

**Response:**

```json
{
  "id": "770e8400-e29b-41d4-a716-446655440000",
  "title": "Vacation 2024",
  "description": "Summer trip to the mountains",
  "parent_id": null,
  "media_count": 0,
  "created_at": "2024-12-16T11:00:00Z"
}
```

---

### List Albums

```http
GET /api/v1/albums
```

**Response:**

```json
{
  "albums": [
    {
      "id": "uuid",
      "title": "Vacation 2024",
      "media_count": 156,
      "cover_thumbnail_url": "/api/v1/media/uuid/items/thumb-uuid",
      "created_at": "2024-12-16T11:00:00Z"
    }
  ]
}
```

---

### Get Album

```http
GET /api/v1/albums/{album_id}
```

---

### Update Album

```http
PUT /api/v1/albums/{album_id}
```

---

### Delete Album

```http
DELETE /api/v1/albums/{album_id}
```

---

### Get Album Media

```http
GET /api/v1/albums/{album_id}/media
```

---

## System Endpoints

### Health Check

```http
GET /api/v1/health
```

**Description:** Health check endpoint (no authentication required).

**Response:**

```json
{
  "status": "healthy",
  "version": "1.0.0",
  "timestamp": "2024-12-16T10:00:00Z",
  "components": {
    "database": "healthy",
    "storage": "healthy",
    "event_bus": "healthy"
  }
}
```

**Status Codes:**

- `200 OK` - System healthy
- `503 Service Unavailable` - System unhealthy

---

### System Statistics (Admin)

```http
GET /api/v1/system/stats
```

**Description:** System-wide statistics (admin only).

**Authentication:** Required (admin role)

**Response:**

```json
{
  "users": {
    "total": 1523,
    "active_today": 342
  },
  "media": {
    "total": 245678,
    "by_type": {
      "Photo": 230000,
      "Video": 15000,
      "Other": 678
    },
    "by_state": {
      "Ready": 240000,
      "Processing": 120,
      "Failed": 5558
    }
  },
  "storage": {
    "total_bytes": 5497558138880,
    "used_bytes": 3298534883328,
    "by_tier": {
      "Permanent": 3000000000000,
      "Cache": 298534883328,
      "Temporary": 0
    }
  },
  "processing": {
    "avg_time_ms": 18500,
    "success_rate": 0.98
  }
}
```

**Status Codes:**

- `200 OK` - Success
- `403 Forbidden` - Not admin

---

## OpenAPI Specification

Full OpenAPI 3.0 specification available at:

```http
GET /api/v1/openapi.json
GET /api/v1/openapi.yaml
```

Interactive API documentation:

```http
GET /api-docs
```

---

## Summary

### Endpoint Summary

**User Endpoints (2):**

- `GET /api/v1/users/me`
- `GET /api/v1/users/me/quota`

**Media Upload (1):**

- `POST /api/v1/media`

**Media Retrieval (3):**

- `GET /api/v1/media/{id}`
- `GET /api/v1/media/{id}/items/{item_id}`
- `GET /api/v1/media`
- `GET /api/v1/media/search`

**Media Management (5):**

- `DELETE /api/v1/media/{id}`
- `POST /api/v1/media/{id}/tags`
- `DELETE /api/v1/media/{id}/tags`
- `PUT /api/v1/media/{id}/album`
- `DELETE /api/v1/media/{id}/album`

**Albums (5 - future):**

- `POST /api/v1/albums`
- `GET /api/v1/albums`
- `GET /api/v1/albums/{id}`
- `PUT /api/v1/albums/{id}`
- `DELETE /api/v1/albums/{id}`
- `GET /api/v1/albums/{id}/media`

**System (2):**

- `GET /api/v1/health`
- `GET /api/v1/system/stats`

**Total: 18 endpoints**

### API Design Principles

✅ **RESTful** - Standard HTTP methods and status codes

✅ **Consistent** - Uniform request/response formats

✅ **Paginated** - Large result sets use pagination

✅ **Filterable** - Flexible filtering and sorting

✅ **Versioned** - API versioning via URL prefix

✅ **Documented** - OpenAPI specification

✅ **Secure** - JWT authentication, HTTPS

✅ **Observable** - Request IDs, tracing headers