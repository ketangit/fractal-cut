package com.puzzlecore.controller.dto;

import java.util.UUID;

public record GenerateResponse(
    UUID id,
    String svgPreview,
    int pieceCount,
    double widthMm,
    double heightMm
) {}
