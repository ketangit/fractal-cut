package com.puzzlecore.model;

import jakarta.persistence.*;
import java.math.BigDecimal;
import java.time.OffsetDateTime;
import java.util.UUID;

@Entity
@Table(name = "puzzle_configs")
public class PuzzleConfig {

    @Id
    @GeneratedValue(strategy = GenerationType.UUID)
    private UUID id;

    @Column(nullable = false)
    private int seed;

    @Column(nullable = false)
    private short ncols;

    @Column(nullable = false)
    private short nrows;

    @Column(name = "min_piece", nullable = false)
    private short minPiece;

    @Column(name = "max_piece", nullable = false)
    private short maxPiece;

    @Column(name = "tile_radius", nullable = false, precision = 8, scale = 4)
    private BigDecimal tileRadius;

    @Column(nullable = false, precision = 8, scale = 4)
    private BigDecimal frame;

    @Column(name = "frame_corner", nullable = false, precision = 8, scale = 4)
    private BigDecimal frameCorner;

    @Column(name = "arc_shape", nullable = false)
    private short arcShape = 0;

    @Column(name = "border_svg", columnDefinition = "TEXT")
    private String borderSvg;

    @Column(name = "border_scale", nullable = false, precision = 8, scale = 4)
    private BigDecimal borderScale = BigDecimal.ONE;

    @Column(name = "created_at", nullable = false, updatable = false)
    private OffsetDateTime createdAt = OffsetDateTime.now();

    public UUID getId() { return id; }
    public int getSeed() { return seed; }
    public void setSeed(int seed) { this.seed = seed; }
    public short getNcols() { return ncols; }
    public void setNcols(short ncols) { this.ncols = ncols; }
    public short getNrows() { return nrows; }
    public void setNrows(short nrows) { this.nrows = nrows; }
    public short getMinPiece() { return minPiece; }
    public void setMinPiece(short minPiece) { this.minPiece = minPiece; }
    public short getMaxPiece() { return maxPiece; }
    public void setMaxPiece(short maxPiece) { this.maxPiece = maxPiece; }
    public BigDecimal getTileRadius() { return tileRadius; }
    public void setTileRadius(BigDecimal tileRadius) { this.tileRadius = tileRadius; }
    public BigDecimal getFrame() { return frame; }
    public void setFrame(BigDecimal frame) { this.frame = frame; }
    public BigDecimal getFrameCorner() { return frameCorner; }
    public void setFrameCorner(BigDecimal frameCorner) { this.frameCorner = frameCorner; }
    public short getArcShape() { return arcShape; }
    public void setArcShape(short arcShape) { this.arcShape = arcShape; }
    public String getBorderSvg() { return borderSvg; }
    public void setBorderSvg(String borderSvg) { this.borderSvg = borderSvg; }
    public BigDecimal getBorderScale() { return borderScale; }
    public void setBorderScale(BigDecimal borderScale) { this.borderScale = borderScale; }
    public OffsetDateTime getCreatedAt() { return createdAt; }
}
