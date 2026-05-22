package com.puzzlecore.controller;

import com.puzzlecore.controller.dto.OrderRequest;
import com.puzzlecore.controller.dto.OrderResponse;
import com.puzzlecore.model.Order;
import com.puzzlecore.model.PuzzleConfig;
import com.puzzlecore.repository.OrderRepository;
import com.puzzlecore.repository.PuzzleConfigRepository;
import jakarta.validation.Valid;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.*;

import java.util.UUID;

@RestController
@RequestMapping("/api/orders")
public class OrderController {

    private final OrderRepository orderRepo;
    private final PuzzleConfigRepository configRepo;

    public OrderController(OrderRepository orderRepo, PuzzleConfigRepository configRepo) {
        this.orderRepo = orderRepo;
        this.configRepo = configRepo;
    }

    @PostMapping
    public ResponseEntity<OrderResponse> create(@Valid @RequestBody OrderRequest req) {
        PuzzleConfig cfg = configRepo.findById(req.configId())
            .orElseThrow(() -> new IllegalArgumentException("Config not found: " + req.configId()));
        Order order = new Order();
        order.setConfig(cfg);
        order = orderRepo.save(order);
        return ResponseEntity.ok(toResponse(order));
    }

    @GetMapping("/{id}")
    public ResponseEntity<OrderResponse> get(@PathVariable UUID id) {
        return orderRepo.findById(id)
            .map(o -> ResponseEntity.ok(toResponse(o)))
            .orElse(ResponseEntity.notFound().build());
    }

    private static OrderResponse toResponse(Order o) {
        return new OrderResponse(o.getId(), o.getConfig().getId(), o.getStatus(), o.getCreatedAt());
    }
}
