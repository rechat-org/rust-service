# Usage Tracking Middleware

## Overview

The Usage Tracker middleware is responsible for tracking and recording API usage for each organization in your system.
It works asynchronously to avoid impacting API response times and integrates with Stripe for usage-based billing.

## Features

- Asynchronous tracking (non-blocking)
- Stripe integration for usage-based billing
- Database usage recording
- Organization-specific tracking

## Implementation Details

### Components

1. **Middleware Implementation**: Implements `FromRequestParts` for seamless integration with Axum
2. **Background Processing**: Uses Tokio's spawn for non-blocking usage tracking
3. **Error Handling**: Comprehensive error logging for tracking failures

### Usage Flow

1. Extracts organization ID from request
2. Spawns background task for tracking
3. Updates both Stripe and local database
4. Returns immediately without blocking the request

### Important Notes

- Should only be applied to endpoints where you want to track API usage
- Does not block or slow down the main request
- Failures in tracking are logged but don't affect the API response
