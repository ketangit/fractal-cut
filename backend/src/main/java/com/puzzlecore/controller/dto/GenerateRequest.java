package com.puzzlecore.controller.dto;

import jakarta.validation.constraints.Max;
import jakarta.validation.constraints.Min;
import jakarta.validation.constraints.NotNull;

public record GenerateRequest(
    @Min(0) @Max(Integer.MAX_VALUE) int seed,
    @Min(2) @Max(100) int ncols,
    @Min(2) @Max(100) int nrows,
    @Min(1) @Max(20) int minPiece,
    @Min(1) @Max(200) int maxPiece,
    @NotNull @Min(2) @Max(20) double tileRadius,
    @Min(0) @Max(20) double frame,
    @Min(0) @Max(10) double frameCorner,
    @Min(0) @Max(2) int arcShape,
    String borderSvg
) {}
