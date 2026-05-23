package com.puzzlecore.controller.dto;

import jakarta.validation.constraints.DecimalMax;
import jakarta.validation.constraints.DecimalMin;
import jakarta.validation.constraints.Max;
import jakarta.validation.constraints.Min;

public record GenerateRequest(
    @Min(0) @Max(Integer.MAX_VALUE) int seed,
    @Min(2) @Max(100) int ncols,
    @Min(2) @Max(100) int nrows,
    @Min(1) @Max(20) int minPiece,
    @Min(1) @Max(200) int maxPiece,
    @DecimalMin("2.0") @DecimalMax("20.0") double tileRadius,
    @DecimalMin("0.0") @DecimalMax("20.0") double frame,
    @DecimalMin("0.0") @DecimalMax("10.0") double frameCorner,
    @Min(0) @Max(2) int arcShape,
    String borderSvg
) {}
