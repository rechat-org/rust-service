# Usage Limiter Middleware

## Overview

The Usage Limiter middleware enforces API usage limits based on organization tiers. It provides real-time usage checking
and limit enforcement, with proactive notifications for approaching limits.

## Features

- Real-time usage limit enforcement
- Tier-based limiting
- Proactive threshold notifications (at 80% usage)
- Redis caching for performance
- Stripe integration for usage data

## Implementation Details

### Components

1. **Usage Checking**: Real-time verification against limits
2. **Cache Integration**: Redis-first approach for performance
3. **Threshold Notifications**: Background processing for usage warnings
4. **Tier Management**: Database integration for tier limits

### Flow

1. Extracts organization ID
2. Checks current usage (Redis/Stripe)
3. Verifies against tier limits
4. Sends notifications if approaching limit
5. Blocks requests if limit exceeded

## How to Use

### Basic Implementation

### Important Notes

- Should be applied globally to enforce limits across all endpoints
- Does not increment usage counters (separate from tracking)
- Cached results for performance
- Asynchronous notification system

### Configuration Requirements

- Redis connection for caching
- Database access for tier information
- Stripe configuration for usage data
- Notification system configuration

## Error Handling

### Types of Errors

- `UsageLimitExceeded`: When organization exceeds their tier limit
- `DatabaseError`: Issues with database operations
- `CacheError`: Redis-related issues
- `NotFound`: Organization or tier not found

### Error Response Example

```json
{
  "error": "Usage limit exceeded. Current usage: 1050, Tier limit: 1000. Please upgrade your subscription."
}
```

## Best Practices

1. Always use in combination with Usage Tracker for accurate tracking
2. Configure appropriate notification thresholds
3. Regular cache invalidation for usage data
4. Monitor error logs for tracking failures
5. Set up alerts for repeated limit violations

## Architecture Considerations

- Middleware order matters - UsageLimiter should run before UsageTracker
- Cache TTL should align with your billing cycle
- Consider implementing retry logic for failed Stripe operations
- Plan for cache failures with fallback to Stripe

Remember that these middlewares work together but serve different purposes:

- UsageLimiter: Enforces limits (read-only)
- UsageTracker: Records usage (write operations)