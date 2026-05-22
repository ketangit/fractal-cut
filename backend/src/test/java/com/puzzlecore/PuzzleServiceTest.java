package com.puzzlecore;

import com.puzzlecore.model.PuzzleConfig;
import com.puzzlecore.service.PuzzleService;
import org.junit.jupiter.api.Test;

import java.math.BigDecimal;

import static org.junit.jupiter.api.Assertions.*;

class PuzzleServiceTest {

    @Test
    void toJson_roundtrips_basic_config() {
        PuzzleConfig cfg = new PuzzleConfig();
        cfg.setSeed(42);
        cfg.setNcols((short) 10);
        cfg.setNrows((short) 10);
        cfg.setMinPiece((short) 4);
        cfg.setMaxPiece((short) 50);
        cfg.setTileRadius(BigDecimal.valueOf(6));
        cfg.setFrame(BigDecimal.valueOf(6));
        cfg.setFrameCorner(BigDecimal.valueOf(4));
        cfg.setArcShape((short) 0);

        String json = PuzzleService.toJson(cfg, null);

        assertTrue(json.contains("\"seed\":42"));
        assertTrue(json.contains("\"ncols\":10"));
        assertTrue(json.contains("\"nrows\":10"));
        assertTrue(json.contains("\"min_piece\":4"));
        assertTrue(json.contains("\"max_piece\":50"));
        assertTrue(json.contains("\"tile_radius\":6"));
        assertTrue(json.contains("\"frame\":6"));
        assertTrue(json.contains("\"frame_corner\":4"));
        assertTrue(json.contains("\"arc_shape\":0"));
        assertFalse(json.contains("custom_border_svg"));
    }

    @Test
    void toJson_includes_border_when_present() {
        PuzzleConfig cfg = new PuzzleConfig();
        cfg.setSeed(1);
        cfg.setNcols((short) 5);
        cfg.setNrows((short) 5);
        cfg.setMinPiece((short) 1);
        cfg.setMaxPiece((short) 25);
        cfg.setTileRadius(BigDecimal.valueOf(4));
        cfg.setFrame(BigDecimal.valueOf(4));
        cfg.setFrameCorner(BigDecimal.valueOf(2));
        cfg.setArcShape((short) 0);
        cfg.setBorderSvg("<svg>circle</svg>");
        cfg.setBorderScale(BigDecimal.ONE);

        String json = PuzzleService.toJson(cfg, null);

        assertTrue(json.contains("custom_border_svg"));
        assertTrue(json.contains("border_scale"));
    }
}
