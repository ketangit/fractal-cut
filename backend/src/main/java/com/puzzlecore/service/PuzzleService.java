package com.puzzlecore.service;

import com.puzzlecore.controller.dto.GenerateRequest;
import com.puzzlecore.controller.dto.GenerateResponse;
import com.puzzlecore.model.PuzzleConfig;
import com.puzzlecore.model.PuzzleOutput;
import com.puzzlecore.repository.PuzzleConfigRepository;
import com.puzzlecore.repository.PuzzleOutputRepository;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.math.BigDecimal;
import java.util.Optional;
import java.util.UUID;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

@Service
public class PuzzleService {

    private static final Pattern PIECE_COUNT_PATTERN =
        Pattern.compile("data-piece-count=\"(\\d+)\"");

    private final PuzzleWasmService wasmService;
    private final PuzzleConfigRepository configRepo;
    private final PuzzleOutputRepository outputRepo;

    public PuzzleService(PuzzleWasmService wasmService,
                         PuzzleConfigRepository configRepo,
                         PuzzleOutputRepository outputRepo) {
        this.wasmService = wasmService;
        this.configRepo = configRepo;
        this.outputRepo = outputRepo;
    }

    @Transactional
    public GenerateResponse generate(GenerateRequest req) {
        PuzzleConfig cfg = toEntity(req);
        cfg = configRepo.save(cfg);

        // Mode 3 = colored preview
        String configJson = toJson(cfg, null);
        String svgPreview = wasmService.generate(configJson, 3);
        int pieceCount = extractPieceCount(svgPreview);

        PuzzleOutput out = new PuzzleOutput();
        out.setConfig(cfg);
        out.setMode((short) 3);
        out.setSvgData(svgPreview);
        out.setPieceCount(pieceCount);
        out.setWasmSha256(wasmService.getWasmSha256());
        outputRepo.save(out);

        double widthMm = cfg.getNcols() * 2.0 * cfg.getTileRadius().doubleValue()
            + 2.0 * cfg.getFrame().doubleValue();
        double heightMm = cfg.getNrows() * 2.0 * cfg.getTileRadius().doubleValue()
            + 2.0 * cfg.getFrame().doubleValue();

        return new GenerateResponse(cfg.getId(), svgPreview, pieceCount, widthMm, heightMm);
    }

    @Transactional
    public String export(UUID configId, int mode) {
        Optional<PuzzleOutput> cached = outputRepo.findByConfigIdAndMode(configId, (short) mode);
        if (cached.isPresent()) {
            return cached.get().getSvgData();
        }

        PuzzleConfig cfg = configRepo.findById(configId)
            .orElseThrow(() -> new IllegalArgumentException("Config not found: " + configId));

        String configJson = toJson(cfg, null);
        String svg = wasmService.generate(configJson, mode);

        PuzzleOutput out = new PuzzleOutput();
        out.setConfig(cfg);
        out.setMode((short) mode);
        out.setSvgData(svg);
        out.setPieceCount(extractPieceCount(svg));
        out.setWasmSha256(wasmService.getWasmSha256());
        outputRepo.save(out);

        return svg;
    }

    private static int extractPieceCount(String svg) {
        Matcher m = PIECE_COUNT_PATTERN.matcher(svg);
        return m.find() ? Integer.parseInt(m.group(1)) : 0;
    }

    private static PuzzleConfig toEntity(GenerateRequest req) {
        PuzzleConfig cfg = new PuzzleConfig();
        cfg.setSeed(req.seed());
        cfg.setNcols((short) req.ncols());
        cfg.setNrows((short) req.nrows());
        cfg.setMinPiece((short) req.minPiece());
        cfg.setMaxPiece((short) req.maxPiece());
        cfg.setTileRadius(BigDecimal.valueOf(req.tileRadius()));
        cfg.setFrame(BigDecimal.valueOf(req.frame()));
        cfg.setFrameCorner(BigDecimal.valueOf(req.frameCorner()));
        cfg.setArcShape((short) req.arcShape());
        cfg.setBorderSvg(req.borderSvg());
        return cfg;
    }

    public static String toJson(PuzzleConfig cfg, String borderSvg) {
        StringBuilder sb = new StringBuilder("{");
        sb.append("\"seed\":").append(cfg.getSeed()).append(",");
        sb.append("\"ncols\":").append(cfg.getNcols()).append(",");
        sb.append("\"nrows\":").append(cfg.getNrows()).append(",");
        sb.append("\"min_piece\":").append(cfg.getMinPiece()).append(",");
        sb.append("\"max_piece\":").append(cfg.getMaxPiece()).append(",");
        sb.append("\"tile_radius\":").append(cfg.getTileRadius()).append(",");
        sb.append("\"frame\":").append(cfg.getFrame()).append(",");
        sb.append("\"frame_corner\":").append(cfg.getFrameCorner()).append(",");
        sb.append("\"arc_shape\":").append(cfg.getArcShape());
        String border = borderSvg != null ? borderSvg : cfg.getBorderSvg();
        if (border != null && !border.isBlank()) {
            sb.append(",\"custom_border_svg\":").append(jsonString(border));
            sb.append(",\"border_scale\":").append(cfg.getBorderScale());
        }
        sb.append("}");
        return sb.toString();
    }

    private static String jsonString(String s) {
        return "\"" + s.replace("\\", "\\\\").replace("\"", "\\\"")
            .replace("\n", "\\n").replace("\r", "\\r").replace("\t", "\\t") + "\"";
    }
}
