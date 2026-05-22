package com.puzzlecore.controller.dto;

import jakarta.validation.constraints.NotNull;
import java.util.UUID;

public record OrderRequest(@NotNull UUID configId) {}
