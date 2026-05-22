package com.puzzlecore.model;

import jakarta.persistence.*;
import java.time.OffsetDateTime;
import java.util.UUID;

@Entity
@Table(name = "puzzle_outputs")
public class PuzzleOutput {

    @Id
    @GeneratedValue(strategy = GenerationType.UUID)
    private UUID id;

    @ManyToOne(fetch = FetchType.LAZY, optional = false)
    @JoinColumn(name = "config_id", nullable = false)
    private PuzzleConfig config;

    @Column(nullable = false)
    private short mode;

    @Column(name = "svg_data", nullable = false, columnDefinition = "TEXT")
    private String svgData;

    @Column(name = "piece_count", nullable = false)
    private int pieceCount;

    @Column(name = "wasm_sha256", nullable = false, length = 64)
    private String wasmSha256;

    @Column(name = "created_at", nullable = false, updatable = false)
    private OffsetDateTime createdAt = OffsetDateTime.now();

    public UUID getId() { return id; }
    public PuzzleConfig getConfig() { return config; }
    public void setConfig(PuzzleConfig config) { this.config = config; }
    public short getMode() { return mode; }
    public void setMode(short mode) { this.mode = mode; }
    public String getSvgData() { return svgData; }
    public void setSvgData(String svgData) { this.svgData = svgData; }
    public int getPieceCount() { return pieceCount; }
    public void setPieceCount(int pieceCount) { this.pieceCount = pieceCount; }
    public String getWasmSha256() { return wasmSha256; }
    public void setWasmSha256(String wasmSha256) { this.wasmSha256 = wasmSha256; }
    public OffsetDateTime getCreatedAt() { return createdAt; }
}
