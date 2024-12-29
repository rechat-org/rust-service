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

### Important Notes

- Does not increment usage counters (separate from tracking)
- Cached results for performance
- Asynchronous notification system

### Configuration Requirements

- Redis connection for caching
- Database access for tier information
- Stripe configuration for usage data
- Notification system configuration
