# Photonic Use Cases

This document describes all use cases for the Photonic photo management system, organized by bounded
context.

## Table of Contents

- [User Management](#user-management)
    - [UC-U1: Ensure User Exists](#uc-u1-ensure-user-exists-oauth-first-time-login)
    - [UC-U2: Check Quota Availability](#uc-u2-check-quota-availability)
    - [UC-U3: Reserve Quota](#uc-u3-reserve-quota)
    - [UC-U4: Commit Quota Reservation](#uc-u4-commit-quota-reservation)
    - [UC-U5: Release Quota Reservation](#uc-u5-release-quota-reservation)
- [Medium Upload and Processing](#medium-upload-and-processing)
    - [UC-M1: Upload Medium Stream](#uc-m1-upload-medium-stream)
    - [UC-M2: Extract EXIF Metadata](#uc-m2-extract-exif-metadata)
    - [UC-M3: Move Medium to Final Location](#uc-m3-move-medium-to-final-location)
    - [UC-M4: Generate Thumbnail](#uc-m4-generate-thumbnail)
    - [UC-M5: Generate Preview Image](#uc-m5-generate-preview-image)
    - [UC-M6: Generate Low-Res Image](#uc-m6-generate-low-res-image-for-raw-files)
    - [UC-M7: Perform Image Recognition](#uc-m7-perform-image-recognition-future)
    - [UC-M8: Mark Medium as Ready](#uc-m8-mark-medium-as-ready)
    - [UC-M9: Handle Processing Failure](#uc-m9-handle-processing-failure)
- [Medium Retrieval](#medium-retrieval)
    - [UC-M10: Get Medium Metadata](#uc-m10-get-medium-metadata)
    - [UC-M11: Get Medium Item](#uc-m11-get-medium-item-download-variant)
    - [UC-M12: List User Media](#uc-m12-list-user-media)
    - [UC-M13: Search Media](#uc-m13-search-media)
- [Medium Management](#medium-management)
    - [UC-M14: Delete Medium](#uc-m14-delete-medium)
    - [UC-M15: Add Tags to Medium](#uc-m15-add-tags-to-medium)
    - [UC-M16: Remove Tags from Medium](#uc-m16-remove-tags-from-medium)
    - [UC-M17: Assign Medium to Album](#uc-m17-assign-medium-to-album)
- [Album Management](#album-management)

---

## User Management

### UC-U1: Ensure User Exists (OAuth First-Time Login)

**Actor:** System (triggered by OAuth callback)

**Preconditions:** Valid OAuth token with user claims

**Postconditions:** User exists in system with default quota

**Main Flow:**

1. System receives OAuth token with `user_id`, `username`, `email`, `quota` from Identity Provider
2. System checks if user exists by `user_id`
3. If user does not exist:
    - Create new user record with provided details
    - Initialize `used_storage` to 0
    - Initialize `reserved_storage` to 0
    - Publish `UserCreated` event
4. If user exists:
    - Update `username` and `email` if they have changed
    - Update `quota` if it has changed by IDP
    - Publish `UserQuotaUpdated` event (if quota changed)
5. Return user entity

**Events Published:**

- `UserCreated` (on new user)
- `UserQuotaUpdated` (if quota changed)

**Alternative Flows:**

- Invalid token → Return authentication error

---

### UC-U2: Check Quota Availability

**Actor:** User

**Preconditions:** User authenticated

**Postconditions:** Returns available quota in bytes

**Main Flow:**

1. Retrieve user's `quota` and `used_storage` from repository
2. Calculate available quota: `available = quota - used_storage - reserved_storage`
3. Return available storage amount

**API Response:**

```json
{
  "quota_bytes": 10737418240,
  "used_bytes": 8388608,
  "reserved_bytes": 1048576,
  "available_bytes": 10727980656
}
```

---

### UC-U3: Reserve Quota

**Actor:** System (during upload)

**Preconditions:**

- User authenticated
- File size known

**Postconditions:** Quota reserved or error if insufficient

**Main Flow:**

1. Retrieve user's current quota status
2. Calculate available quota: `available = quota - used_storage - reserved_storage`
3. If `file_size > available`:
    - Publish `UserQuotaExceeded` event
    - Return `QuotaExceededError`
4. Create quota reservation record with:
    - `reservation_id` (UUID)
    - `user_id`
    - `reserved_bytes`: file_size
    - `created_at`: current timestamp
    - `expires_at`: current timestamp + 1 hour
    - `status`: "Active"
5. Increment user's `reserved_storage` by file_size (atomic update)
6. Publish `QuotaReserved` event
7. Return `reservation_id`

**Events Published:**

- `QuotaReserved` (on success)
- `UserQuotaExceeded` (on insufficient quota)

**Alternative Flows:**

- Insufficient quota → Return HTTP 413 Payload Too Large with error details

**Error Response:**

```json
{
  "error": "quota_exceeded",
  "message": "Insufficient quota. Required: 8388608 bytes, Available: 1048576 bytes",
  "quota_available": 1048576
}
```

---

### UC-U4: Commit Quota Reservation

**Actor:** System (after successful processing)

**Preconditions:** Valid `reservation_id`

**Postconditions:** Quota permanently allocated to used storage

**Main Flow:**

1. Find reservation record by `reservation_id`
2. If reservation not found or status != "Active":
    - Log warning
    - Return error
3. Atomically update user record:
    - Increment `used_storage` by `reserved_bytes`
    - Decrement `reserved_storage` by `reserved_bytes`
4. Update reservation status to "Committed"
5. Publish `QuotaCommitted` event
6. Return success

**Events Published:**

- `QuotaCommitted`

**Alternative Flows:**

- Reservation not found → Log error and continue (idempotent)
- Reservation already committed → Return success (idempotent)

---

### UC-U5: Release Quota Reservation

**Actor:** System (on upload failure or timeout)

**Preconditions:** Valid `reservation_id`

**Postconditions:** Quota reservation released

**Main Flow:**

1. Find reservation record by `reservation_id`
2. If reservation not found or status != "Active":
    - Log warning
    - Return success (idempotent)
3. Atomically decrement user's `reserved_storage` by `reserved_bytes`
4. Update reservation status to "Released"
5. Publish `QuotaReleased` event with reason
6. Return success

**Events Published:**

- `QuotaReleased`

**Release Reasons:**

- `UploadFailed` - Upload operation failed
- `ProcessingFailed` - Processing pipeline failed
- `Timeout` - Reservation expired (cleanup job)
- `UserCancelled` - User cancelled upload

---

## Medium Upload and Processing

### UC-M1: Upload Medium Stream

**Actor:** User

**Preconditions:**

- User authenticated with valid JWT token
- File to upload available as stream

**Postconditions:**

- File uploaded to temporary storage
- Medium entity created
- Processing pipeline started

**Main Flow:**

1. User sends POST request to `/api/v1/media` with:
    - File stream (multipart/form-data)
    - File size (Content-Length header)
    - MIME type
    - Original filename
    - Optional metadata: tags, album_id, priority
2. System calls UC-U3 (Reserve Quota) with file_size
    - If fails → Return 413 error to user
3. System generates `medium_id` (UUID)
4. System determines file extension from filename
5. System creates temporary storage location: `temp/{medium_id}.{ext}`
6. System creates Medium entity:
    - `id`: medium_id
    - `user_id`: from JWT
    - `state`: "Uploading"
    - `medium_type`: derived from MIME type
    - `original_filename`: filename
    - `metadata`: empty
    - `tags`: from request
    - `album_id`: from request (if provided)
7. **Execute in parallel:**
    - **Task A:** Stream file to temporary storage via FileStorage port
    - **Task B:** Save Medium entity to repository
8. **Handle results:**
    - **Both succeed:**
        - Update Medium state to "Processing"
        - Publish `MediumUploaded` event
        - Return 201 Created with medium_id
    - **Storage failed, DB succeeded:**
        - Delete Medium from repository
        - Call UC-U5 (Release Quota)
        - Return 500 error
    - **Storage succeeded, DB failed:**
        - Delete file from temporary storage
        - Call UC-U5 (Release Quota)
        - Return 500 error
    - **Both failed:**
        - Call UC-U5 (Release Quota)
        - Return 500 error

**Events Published:**

- `MediumUploaded` (on success)

**API Request:**

```http
POST /api/v1/media
Content-Type: multipart/form-data
Authorization: Bearer {jwt_token}

--boundary
Content-Disposition: form-data; name="file"; filename="IMG_1234.jpg"
Content-Type: image/jpeg

{binary data}
--boundary
Content-Disposition: form-data; name="tags"

vacation,landscape
--boundary
```

**API Response (Success):**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "status": "task",
  "message": "Upload successful. Processing started.",
  "created_at": "2024-12-16T10:35:00Z"
}
```

**Alternative Flows:**

- Quota exceeded → HTTP 413 with quota details
- Invalid file type → HTTP 400 with validation error
- Upload interrupted → Cleanup and return 500

---

### UC-M2: Extract EXIF Metadata

**Actor:** System (Event Listener)

**Trigger:** `MediumUploaded` event

**Preconditions:** Medium file exists in temporary storage

**Postconditions:** EXIF metadata extracted and stored in Medium entity

**Main Flow:**

1. Event listener receives `MediumUploaded` event
2. Retrieve Medium entity from repository
3. Call `MetadataExtractor.extract_metadata(temp_location)`
4. MetadataExtractor executes exiftool on file and parses output
5. Extract relevant EXIF fields:
    - `taken_at` (DateTimeOriginal)
    - `camera_make` (Make)
    - `camera_model` (Model)
    - `lens_model` (LensModel)
    - `iso` (ISO)
    - `aperture` (FNumber)
    - `shutter_speed` (ExposureTime)
    - `focal_length` (FocalLength)
    - `gps_coordinates` (GPSLatitude, GPSLongitude)
6. Update Medium entity with extracted metadata
7. Save updated Medium to repository
8. Publish `MediumMetadataExtracted` event

**Events Published:**

- `MediumMetadataExtracted` (on success)
- `MediumProcessingFailed` (on critical failure)

**Alternative Flows:**

- **EXIF extraction fails:**
    - Check if metadata required for storage path pattern
    - If required fields missing (e.g., date for `{year}/{month}` pattern):
        - Update Medium state to "Failed"
        - Publish `MediumProcessingFailed` event
        - Stop processing
    - If only optional fields missing:
        - Use fallback values (upload timestamp for date, "Unknown" for camera)
        - Continue with partial metadata
- **File corrupted or unreadable:**
    - Publish `MediumProcessingFailed` event
    - Mark as permanently failed

**Example Metadata:**

```json
{
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
}
```

---

### UC-M3: Move Medium to Final Location

**Actor:** System (Event Listener)

**Trigger:** `MediumMetadataExtracted` event

**Preconditions:**

- Metadata extracted
- Required metadata fields present

**Postconditions:**

- Medium moved from temporary to permanent storage
- MediumItem created for original file
- Quota reservation committed

**Main Flow:**

1. Event listener receives `MediumMetadataExtracted` event
2. Retrieve Medium entity from repository
3. Retrieve storage path pattern from configuration (e.g.,
   `{year}/{month}/{camera_make}/{filename}`)
4. Call `StoragePathService.calculate_path(pattern, metadata, filename)`
    - Replace `{year}` with year from `taken_at`
    - Replace `{month}` with month from `taken_at` (zero-padded)
    - Replace `{camera_make}` with sanitized camera make
    - Replace `{filename}` with sanitized original filename
    - Example result: `2024/12/Canon/IMG_1234.jpg`
5. Create permanent storage location: `permanent/{calculated_path}`
6. Move file from temporary to permanent storage
7. Generate `item_id` (UUID)
8. Create MediumItem entity:
    - `id`: item_id
    - `medium_id`: from event
    - `item_type`: "Original"
    - `storage_location`: permanent location
    - `mime_type`: from Medium
    - `file_size`: actual file size
    - `dimensions`: extracted from file (if image)
    - `state`: "Ready"
9. Set `Medium.leading_item_id` to new item_id
10. Add MediumItem to Medium
11. Save Medium with new item
12. Call UC-U4 (Commit Quota Reservation)
13. Delete temporary file
14. Publish `MediumMovedToFinal` event

**Events Published:**

- `MediumMovedToFinal` (on success)
- `MediumProcessingFailed` (on failure)

**Alternative Flows:**

- **Path calculation fails (missing required metadata):**
    - Use fallback values or upload timestamp
    - If fallback not acceptable:
        - Publish `MediumProcessingFailed` event
        - Keep file in temporary storage
- **Move operation fails (disk full, permissions):**
    - Publish `MediumProcessingFailed` event
    - Keep file in temporary storage
    - Release quota reservation
- **Temporary file deletion fails:**
    - Log warning (file will be cleaned up by scheduled job)
    - Continue processing (not critical)

**Example Path Calculation:**

```
Pattern: {year}/{month}/{camera_make}/{filename}
Metadata:
  - taken_at: 2024-12-15T14:22:00Z
  - camera_make: "Canon"
  - filename: "IMG_1234.jpg"

Result: 2024/12/Canon/IMG_1234.jpg
```

---

### UC-M4: Generate Thumbnail

**Actor:** System (Event Listener)

**Trigger:** `MediumMovedToFinal` event

**Preconditions:** Original file exists in permanent storage

**Postconditions:** Thumbnail variant created and stored

**Main Flow:**

1. Event listener receives `MediumMovedToFinal` event
2. Retrieve Medium entity from repository
3. Get original file location from leading_item
4. Load original file stream from storage
5. Resize image to thumbnail dimensions (e.g., 200×200 pixels):
    - Maintain aspect ratio
    - Apply smart cropping if needed
    - Convert to JPEG format
    - Apply compression (quality: 85)
6. Generate `item_id` (UUID)
7. Calculate thumbnail storage path: `cache/{medium_id}_thumb.jpg`
8. Store thumbnail file to cache tier
9. Create MediumItem entity:
    - `id`: item_id
    - `medium_id`: from event
    - `item_type`: "Thumbnail"
    - `storage_location`: cache location
    - `mime_type`: "image/jpeg"
    - `file_size`: actual thumbnail size
    - `dimensions`: 200×200
    - `state`: "Ready"
10. Add MediumItem to Medium
11. Save Medium
12. Publish `ThumbnailCreated` event

**Events Published:**

- `ThumbnailCreated` (on success)

**Alternative Flows:**

- **Image processing fails:**
    - Log error with details
    - Do NOT publish failure event (thumbnail is optional)
    - Continue with other processing
- **Storage fails:**
    - Log error
    - Retry once
    - If retry fails, skip thumbnail creation

**Processing Details:**

- Supported formats: JPEG, PNG, HEIC, RAW (using embedded preview)
- For RAW files: Extract embedded JPEG preview if available
- For videos: Extract frame at 10% duration
- Target size: 200×200 pixels (configurable)
- Output format: JPEG with 85% quality

**Can run in parallel with:** UC-M5, UC-M6

---

### UC-M5: Generate Preview Image

**Actor:** System (Event Listener)

**Trigger:** `MediumMovedToFinal` event

**Preconditions:** Original file exists in permanent storage

**Postconditions:** Preview variant created and stored

**Main Flow:**

1. Event listener receives `MediumMovedToFinal` event
2. Retrieve Medium entity from repository
3. Get original file location from leading_item
4. Load original file stream from storage
5. Resize image to preview dimensions (e.g., 1024 pixels on longest side):
    - Maintain aspect ratio
    - No cropping
    - Convert to JPEG format
    - Apply compression (quality: 90)
6. Generate `item_id` (UUID)
7. Calculate preview storage path: `cache/{medium_id}_preview.jpg`
8. Store preview file to cache tier
9. Create MediumItem entity:
    - `id`: item_id
    - `medium_id`: from event
    - `item_type`: "Preview"
    - `storage_location`: cache location
    - `mime_type`: "image/jpeg"
    - `file_size`: actual preview size
    - `dimensions`: e.g., 1024×768
    - `state`: "Ready"
10. Add MediumItem to Medium
11. Save Medium
12. Publish `PreviewCreated` event

**Events Published:**

- `PreviewCreated` (on success)

**Alternative Flows:**

- Similar error handling to UC-M4

**Processing Details:**

- Target size: 1024 pixels on longest side (configurable)
- Output format: JPEG with 90% quality
- Used for: Web display, quick viewing

**Can run in parallel with:** UC-M4, UC-M6

---

### UC-M6: Generate Low-Res Image (for RAW files)

**Actor:** System (Event Listener)

**Trigger:** `MediumMovedToFinal` event

**Preconditions:**

- Original file exists in permanent storage
- Medium type is RAW format

**Postconditions:** Low-res JPEG variant created

**Main Flow:**

1. Event listener receives `MediumMovedToFinal` event
2. Retrieve Medium entity from repository
3. Check if `mime_type` is RAW format:
    - Canon: image/x-canon-cr2, image/x-canon-cr3
    - Nikon: image/x-nikon-nef
    - Sony: image/x-sony-arw
    - Adobe: image/x-adobe-dng
4. If not RAW → Skip this use case
5. Get original file location from leading_item
6. Convert RAW to JPEG using RAW processor:
    - Use libraw or similar library
    - Apply default white balance
    - Resize to low-res dimensions (e.g., 2048 pixels)
    - Output format: JPEG with 92% quality
7. Generate `item_id` (UUID)
8. Calculate low-res storage path: `cache/{medium_id}_lowres.jpg`
9. Store low-res file to cache tier
10. Create MediumItem entity:
    - `id`: item_id
    - `medium_id`: from event
    - `item_type`: "LowRes"
    - `storage_location`: cache location
    - `mime_type`: "image/jpeg"
    - `file_size`: actual size
    - `dimensions`: e.g., 2048×1365
    - `state`: "Ready"
11. Add MediumItem to Medium
12. Save Medium
13. Publish `LowResCreated` event

**Events Published:**

- `LowResCreated` (on success)

**Alternative Flows:**

- **RAW conversion fails:**
    - Try extracting embedded preview instead
    - If that also fails, log error and skip
- **Not a RAW file:**
    - Skip processing (normal behavior)

**Processing Details:**

- Target size: 2048 pixels on longest side
- Output format: JPEG with 92% quality
- Used for: Faster viewing than original RAW, input for image recognition

**Can run in parallel with:** UC-M4, UC-M5

---

### UC-M7: Perform Image Recognition (Future)

**Actor:** System (Event Listener)

**Trigger:** `LowResCreated` or `PreviewCreated` event

**Preconditions:** Low-res or preview image available

**Postconditions:** Recognition data stored in Medium metadata

**Main Flow:**

1. Event listener receives `LowResCreated` or `PreviewCreated` event
2. Determine best image for recognition:
    - Prefer LowRes if available (better quality)
    - Fallback to Preview
3. Retrieve Medium entity from repository
4. Load image file from storage
5. Call ML/AI service with image:
    - Face detection with bounding boxes
    - Object detection with labels and confidence
    - Scene classification (landscape, portrait, indoor, etc.)
6. Receive recognition results
7. Update Medium metadata with recognition data:
    - Detected faces: [{ x, y, width, height, confidence }]
    - Objects: [{ label, confidence }]
    - Scene: { label, confidence }
    - Colors: dominant color palette
8. Save updated Medium
9. Publish `ImageRecognitionCompleted` event

**Events Published:**

- `ImageRecognitionCompleted` (on success)

**Alternative Flows:**

- **ML service unavailable:**
    - Retry with exponential backoff
    - If max retries exceeded, log and skip
- **Low confidence results:**
    - Only store results above confidence threshold (e.g., 0.7)

**Example Recognition Data:**

```json
{
  "faces": [
    {"x": 100, "y": 150, "width": 80, "height": 100, "confidence": 0.95}
  ],
  "objects": [
    {"label": "mountain", "confidence": 0.92},
    {"label": "sky", "confidence": 0.88}
  ],
  "scene": {"label": "landscape", "confidence": 0.89},
  "colors": ["#4A90E2", "#7ED321", "#F5A623"]
}
```

---

### UC-M8: Mark Medium as Ready

**Actor:** System (Completion Checker)

**Trigger:** Various processing completion events

**Preconditions:** All required processing steps complete

**Postconditions:** Medium state set to "Ready"

**Main Flow:**

1. After each processing event (`MediumMovedToFinal`, `ThumbnailCreated`, etc.):
    - Retrieve Medium entity
    - Check completion status:
        - **Required:** Original moved to final location (leading_item_id set)
        - **Optional:** Thumbnails, previews, recognition
2. If all required steps complete:
    - Calculate total processing time
    - Update Medium state to "Ready"
    - Record completion timestamp
    - Save Medium
    - Publish `MediumReady` event

**Events Published:**

- `MediumReady`

**Completion Criteria:**

- Minimum: Original file in permanent storage
- Recommended: Original + Thumbnail + Preview
- Full: Original + Thumbnail + Preview + LowRes (if RAW) + Recognition

---

### UC-M9: Handle Processing Failure

**Actor:** System (Error Handler)

**Trigger:** Any processing step encounters error

**Preconditions:** Processing step failed

**Postconditions:** Failure logged and handled appropriately

**Main Flow:**

1. Processing step encounters error (exception, validation failure, external service error)
2. Log detailed error information:
    - Medium ID
    - Processing step that failed
    - Error message and stack trace
    - Timestamp
3. Retrieve Medium entity
4. Determine error type:
    - **Transient errors:** Network timeout, temporary service unavailability
    - **Permanent errors:** Corrupted file, missing required data, validation failure
5. Update Medium entity:
    - State: "Failed"
    - Error details: step name, error message, is_retryable
    - Failed_at timestamp
6. Save Medium
7. Publish `MediumProcessingFailed` event
8. **If transient error:**
    - Schedule retry with exponential backoff
    - Max 3 retries
9. **If permanent error:**
    - Mark as permanently failed
    - Keep file in storage for manual inspection
10. **If retries exhausted:**
    - Update to permanently failed
    - Optionally notify user

**Events Published:**

- `MediumProcessingFailed`

**Error Response (when queried):**

```json
{
  "id": "uuid",
  "state": "Failed",
  "error": {
    "step": "ExifExtraction",
    "message": "Failed to parse EXIF data: corrupt header",
    "is_retryable": false,
    "failed_at": "2024-12-16T10:35:20Z"
  }
}
```

**Cleanup Actions:**

- Release quota reservation if not yet committed
- Keep files for potential manual recovery
- Log for monitoring and alerting

---

## Medium Retrieval

### UC-M10: Get Medium Metadata

**Actor:** User

**Preconditions:**

- User authenticated
- User owns medium or has access

**Postconditions:** Returns complete medium metadata

**Main Flow:**

1. User sends GET request to `/api/v1/media/{medium_id}`
2. Repository fetches Medium by ID and user_id
3. If Medium not found → Return 404 Not Found
4. If Medium.user_id != authenticated user_id → Return 403 Forbidden
5. Return Medium entity as JSON with:
    - Basic info: id, type, filename, state
    - Metadata: EXIF data
    - Items: array of all variants with details
    - Tags
    - Album ID
    - Timestamps

**API Response:**

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
  "tags": ["vacation", "landscape"],
  "album_id": "album-uuid",
  "created_at": "2024-12-16T10:35:00Z",
  "updated_at": "2024-12-16T10:36:00Z"
}
```

---

### UC-M11: Get Medium Item (Download Variant)

**Actor:** User

**Preconditions:**

- User authenticated
- User owns medium or has access

**Postconditions:** Returns file stream of specified variant

**Main Flow:**

1. User sends GET request to `/api/v1/media/{medium_id}/items/{item_id}`
2. Repository fetches Medium by medium_id and user_id
3. Verify ownership (Medium.user_id == authenticated user_id)
4. Find MediumItem by item_id within Medium
5. Check item state:
    - If "Ready" → Continue
    - If "Pending" → Return 202 Accepted with Retry-After header
    - If "Failed" → Return 500 Internal Server Error
6. Get file stream from storage using item.storage_location
7. Return file stream with headers:
    - Content-Type: item.mime_type
    - Content-Length: item.file_size
    - Content-Disposition: inline (for preview) or attachment (for download)
    - Cache-Control: public, max-age=31536000 (cache for 1 year)
    - ETag: hash of file

**API Request:**

```http
GET /api/v1/media/550e8400-e29b-41d4-a716-446655440000/items/item-uuid-1
Authorization: Bearer {jwt_token}
```

**API Response:**

```http
HTTP/1.1 200 OK
Content-Type: image/jpeg
Content-Length: 8388608
Content-Disposition: inline
Cache-Control: public, max-age=31536000
ETag: "abc123def456"

{binary data}
```

**Alternative Flows:**

- **Item still processing (state: Pending):**
  ```http
  HTTP/1.1 202 Accepted
  Retry-After: 30

  {"message": "Item is being processed. Please retry after 30 seconds."}
  ```
- **Item processing failed (state: Failed):**
  ```http
  HTTP/1.1 500 Internal Server Error

  {"error": "variant_generation_failed", "message": "Failed to generate this variant."}
  ```

---

### UC-M12: List User Media

**Actor:** User

**Preconditions:** User authenticated

**Postconditions:** Returns paginated list of user's media

**Main Flow:**

1. User sends GET request to `/api/v1/media` with query parameters:
    - `page`: Page number (default: 1)
    - `limit`: Items per page (default: 20, max: 100)
    - `type`: Filter by medium type ("Photo", "Video", etc.)
    - `album_id`: Filter by album
    - `tags`: Comma-separated tags (OR logic)
    - `date_from`: Filter by taken_at >= date
    - `date_to`: Filter by taken_at <= date
    - `sort`: Sort field and order ("created_at:desc", "taken_at:asc")
    - `state`: Filter by state ("Ready", "Processing", "Failed")
2. Repository queries media table with filters:
    - WHERE user_id = authenticated_user_id
    - AND type = filter_type (if provided)
    - AND album_id = filter_album_id (if provided)
    - AND EXISTS (SELECT 1 FROM medium_tags WHERE tag IN filters) (if tags provided)
    - AND taken_at >= date_from (if provided)
    - AND taken_at <= date_to (if provided)
    - AND state = filter_state (if provided)
    - ORDER BY sort_field sort_order
    - LIMIT limit OFFSET (page - 1) * limit
3. Get total count for pagination
4. For each medium, include:
    - Basic info
    - Thumbnail URL
    - State
    - Tags
5. Return paginated response

**API Request:**

```http
GET /api/v1/media?page=1&limit=20&type=Photo&tags=vacation,landscape&sort=taken_at:desc
Authorization: Bearer {jwt_token}
```

**API Response:**

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
      "dimensions": {"width": 8192, "height": 5464}
    },
    {
      "id": "uuid-2",
      "type": "Photo",
      "filename": "IMG_1235.jpg",
      "thumbnail_url": "/api/v1/media/uuid-2/items/thumb-uuid",
      "taken_at": "2024-12-16T10:28:00Z",
      "state": "Ready",
      "tags": ["vacation"],
      "dimensions": {"width": 8192, "height": 5464}
    }
  ],
  "links": {
    "self": "/api/v1/media?page=1&limit=20",
    "next": "/api/v1/media?page=2&limit=20",
    "last": "/api/v1/media?page=8&limit=20"
  }
}
```

---

### UC-M13: Search Media

**Actor:** User

**Preconditions:** User authenticated

**Postconditions:** Returns matching media based on search query

**Main Flow:**

1. User sends GET request to `/api/v1/media/search` with query parameters:
    - `q`: Search term (required)
    - All filter parameters from UC-M12
2. Repository performs full-text search across:
    - Tags (exact and partial match)
    - Original filename
    - Camera make and model
    - Metadata fields
3. Use PostgreSQL full-text search:
   ```sql
   WHERE user_id = $1
   AND (
     tags @@ to_tsquery($2)
     OR original_filename ILIKE '%' || $2 || '%'
     OR metadata->>'camera_make' ILIKE '%' || $2 || '%'
     OR metadata->>'camera_model' ILIKE '%' || $2 || '%'
   )
   ```
4. Apply additional filters (type, date range, etc.)
5. Return results in same format as UC-M12

**API Request:**

```http
GET /api/v1/media/search?q=canon&page=1&limit=20
Authorization: Bearer {jwt_token}
```

**API Response:**

```json
{
  "query": "canon",
  "total": 45,
  "page": 1,
  "limit": 20,
  "items": [
    {
      "id": "uuid-1",
      "type": "Photo",
      "filename": "IMG_1234.jpg",
      "thumbnail_url": "/api/v1/media/uuid-1/items/thumb-uuid",
      "metadata": {
        "camera_make": "Canon",
        "camera_model": "EOS R5"
      },
      "relevance": 0.95
    }
  ]
}
```

---

## Medium Management

### UC-M14: Delete Medium

**Actor:** User

**Preconditions:**

- User authenticated
- User owns medium

**Postconditions:**

- Medium and all variants deleted from storage
- Medium removed from database
- Quota released

**Main Flow:**

1. User sends DELETE request to `/api/v1/media/{medium_id}`
2. Repository fetches Medium by ID and user_id
3. Verify ownership
4. Calculate total file size of all MediumItems
5. Delete all MediumItems:
    - For each item, delete file from storage
    - Delete item record from database
6. Delete Medium record from repository
7. Update user's used_storage:
    - Decrement by total_file_size
8. Publish `MediumDeleted` event
9. Return 204 No Content

**Events Published:**

- `MediumDeleted`

**API Request:**

```http
DELETE /api/v1/media/550e8400-e29b-41d4-a716-446655440000
Authorization: Bearer {jwt_token}
```

**API Response:**

```http
HTTP/1.1 204 No Content
```

**Alternative Flows:**

- **Storage deletion fails for some files:**
    - Log errors
    - Mark files for cleanup by background job
    - Continue with database deletion
    - Release quota anyway

---

### UC-M15: Add Tags to Medium

**Actor:** User

**Preconditions:**

- User authenticated
- User owns medium

**Postconditions:** Tags added to medium

**Main Flow:**

1. User sends POST request to `/api/v1/media/{medium_id}/tags`
    - Body: `{"tags": ["vacation", "landscape", "summer"]}`
2. Repository fetches Medium by ID and user_id
3. Verify ownership
4. Validate tags:
    - Convert to lowercase
    - Trim whitespace
    - Check length (1-50 chars)
    - Remove duplicates
    - Remove invalid characters
5. Merge new tags with existing tags (set union)
6. For each new tag:
    - Check if tag exists in tags table
    - If not, create tag record
    - Create medium_tags junction record
7. Save updated Medium
8. Publish `MediumTagged` event
9. Return updated Medium with all tags

**Events Published:**

- `MediumTagged`

**API Request:**

```http
POST /api/v1/media/550e8400-e29b-41d4-a716-446655440000/tags
Authorization: Bearer {jwt_token}
Content-Type: application/json

{
  "tags": ["vacation", "landscape", "summer"]
}
```

**API Response:**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "tags": ["vacation", "landscape", "summer", "mountains"],
  "updated_at": "2024-12-16T11:00:00Z"
}
```

**Validation Rules:**

- Tag length: 1-50 characters
- Allowed characters: alphanumeric, hyphen, underscore
- Automatically converted to lowercase
- Max 50 tags per medium

---

### UC-M16: Remove Tags from Medium

**Actor:** User

**Preconditions:**

- User authenticated
- User owns medium

**Postconditions:** Specified tags removed from medium

**Main Flow:**

1. User sends DELETE request to `/api/v1/media/{medium_id}/tags`
    - Body: `{"tags": ["summer"]}`
2. Repository fetches Medium by ID and user_id
3. Verify ownership
4. Remove specified tags from Medium.tags (set difference)
5. Delete corresponding medium_tags junction records
6. Save updated Medium
7. Publish `MediumTagged` event
8. Return updated Medium with remaining tags

**Events Published:**

- `MediumTagged`

**API Request:**

```http
DELETE /api/v1/media/550e8400-e29b-41d4-a716-446655440000/tags
Authorization: Bearer {jwt_token}
Content-Type: application/json

{
  "tags": ["summer"]
}
```

**API Response:**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "tags": ["vacation", "landscape", "mountains"],
  "updated_at": "2024-12-16T11:05:00Z"
}
```

---

### UC-M17: Assign Medium to Album

**Actor:** User

**Preconditions:**

- User authenticated
- User owns both medium and album

**Postconditions:** Medium assigned to album

**Main Flow:**

1. User sends PUT request to `/api/v1/media/{medium_id}/album`
    - Body: `{"album_id": "album-uuid"}`
2. Repository fetches Medium by ID and user_id
3. Verify medium ownership
4. Repository fetches Album by album_id and user_id
5. Verify album ownership
6. Update Medium.album_id to album_id
7. Save Medium
8. Publish `MediumMovedToAlbum` event
9. Return updated Medium

**Events Published:**

- `MediumMovedToAlbum`

**API Request:**

```http
PUT /api/v1/media/550e8400-e29b-41d4-a716-446655440000/album
Authorization: Bearer {jwt_token}
Content-Type: application/json

{
  "album_id": "770e8400-e29b-41d4-a716-446655440000"
}
```

**API Response:**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "album_id": "770e8400-e29b-41d4-a716-446655440000",
  "updated_at": "2024-12-16T11:10:00Z"
}
```

**Alternative Flows:**

- **Album not found or not owned by user:**
    - Return 404 Not Found
- **Remove from album:**
    - Send DELETE to `/api/v1/media/{medium_id}/album`
    - Set album_id to null

---

## Album Management

### UC-A1: Create Album (Future)

**Actor:** User

**Preconditions:** User authenticated

**Postconditions:** Album created

**Main Flow:**

1. User sends POST request to `/api/v1/albums`
    - Body: `{"title": "Vacation 2024", "description": "Summer trip", "parent_id": null}`
2. Validate album title (1-100 chars)
3. Create Album entity:
    - Generate album_id
    - Set user_id from JWT
    - Set title, description, parent_id
4. Save to repository
5. Publish `AlbumCreated` event
6. Return 201 Created with Album

**Events Published:**

- `AlbumCreated`

---

### UC-A2-A5: Additional Album Operations (Future)

- **UC-A2:** Update Album - Modify title, description, cover photo
- **UC-A3:** Delete Album - Remove album (media remain in system)
- **UC-A4:** List Albums - Get user's albums with hierarchy
- **UC-A5:** Get Album Media - List all media in specific album

---

## Summary

This document defines **17 detailed use cases** covering:

- **User Management** (5 use cases) - Authentication, quota management
- **Medium Upload & Processing** (9 use cases) - Upload, EXIF, movement, variant generation
- **Medium Retrieval** (4 use cases) - Get, list, search operations
- **Medium Management** (4 use cases) - Delete, tags, album assignment
- **Album Management** (5 use cases - future)

Each use case includes:

- Actors and triggers
- Preconditions and postconditions
- Detailed main flow with steps
- Alternative flows for error cases
- Events published
- API request/response examples
- Processing details

These use cases form the foundation for implementing the Photonic system's application layer
following the CQRS pattern.