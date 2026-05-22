package com.puzzlecore.controller;

import com.puzzlecore.controller.dto.GenerateRequest;
import com.puzzlecore.controller.dto.GenerateResponse;
import com.puzzlecore.service.PuzzleService;
import jakarta.validation.Valid;
import org.springframework.http.MediaType;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.*;

import java.util.UUID;

@RestController
@RequestMapping("/api/puzzle")
public class PuzzleController {

    private final PuzzleService puzzleService;

    public PuzzleController(PuzzleService puzzleService) {
        this.puzzleService = puzzleService;
    }

    @PostMapping("/generate")
    public ResponseEntity<GenerateResponse> generate(@Valid @RequestBody GenerateRequest req) {
        return ResponseEntity.ok(puzzleService.generate(req));
    }

    @GetMapping("/{id}/export")
    public ResponseEntity<String> export(
        @PathVariable UUID id,
        @RequestParam(defaultValue = "1") int mode
    ) {
        if (mode < 0 || mode > 3) {
            return ResponseEntity.badRequest().build();
        }
        String svg = puzzleService.export(id, mode);
        return ResponseEntity.ok()
            .contentType(MediaType.parseMediaType("image/svg+xml"))
            .header("Content-Disposition",
                "attachment; filename=\"jigsaw-mode" + mode + ".svg\"")
            .body(svg);
    }
}
