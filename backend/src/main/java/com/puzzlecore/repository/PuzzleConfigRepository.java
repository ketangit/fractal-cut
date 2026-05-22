package com.puzzlecore.repository;

import com.puzzlecore.model.PuzzleConfig;
import org.springframework.data.jpa.repository.JpaRepository;
import java.util.UUID;

public interface PuzzleConfigRepository extends JpaRepository<PuzzleConfig, UUID> {}
