package com.puzzlecore.repository;

import com.puzzlecore.model.PuzzleOutput;
import org.springframework.data.jpa.repository.JpaRepository;
import java.util.Optional;
import java.util.UUID;

public interface PuzzleOutputRepository extends JpaRepository<PuzzleOutput, UUID> {
    Optional<PuzzleOutput> findByConfigIdAndMode(UUID configId, short mode);
}
