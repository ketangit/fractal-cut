package com.puzzlecore.controller.dto;

import java.time.OffsetDateTime;
import java.util.UUID;

public record OrderResponse(UUID id, UUID configId, String status, OffsetDateTime createdAt) {}
