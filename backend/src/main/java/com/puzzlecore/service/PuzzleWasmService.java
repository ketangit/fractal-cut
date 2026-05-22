package com.puzzlecore.service;

import com.dylibso.chicory.runtime.ExportFunction;
import com.dylibso.chicory.runtime.Instance;
import com.dylibso.chicory.wasm.Parser;
import com.dylibso.chicory.wasm.WasmModule;
import jakarta.annotation.PostConstruct;
import jakarta.annotation.PreDestroy;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.core.io.ClassPathResource;
import org.springframework.stereotype.Service;

import java.io.IOException;
import java.io.InputStream;
import java.nio.charset.StandardCharsets;
import java.util.concurrent.ArrayBlockingQueue;
import java.util.concurrent.BlockingQueue;

/**
 * Executes the server-side Wasm binary via Chicory.
 *
 * C-ABI exported by the Rust server Wasm:
 *   get_input_ptr()  → i32 (pointer into 64KB input buffer)
 *   get_output_ptr() → i32 (pointer into 4MB output buffer)
 *   generate_puzzle(input_len: i32) → i32 (output_len; 0 = error)
 *
 * Chicory Instance is not thread-safe, so we pool one per available
 * processor bounded by POOL_SIZE to prevent OOM from large SVG buffers.
 */
@Service
public class PuzzleWasmService {

    private static final int POOL_SIZE = Runtime.getRuntime().availableProcessors();

    @Value("${puzzle.wasm.path}")
    private String wasmPath;

    private WasmModule wasmModule;
    private BlockingQueue<Instance> pool;
    private String wasmSha256;

    @PostConstruct
    void init() throws IOException {
        ClassPathResource res = new ClassPathResource(wasmPath);
        byte[] wasmBytes;
        try (InputStream in = res.getInputStream()) {
            wasmBytes = in.readAllBytes();
        }
        wasmSha256 = sha256hex(wasmBytes);
        wasmModule = Parser.parse(wasmBytes);

        pool = new ArrayBlockingQueue<>(POOL_SIZE);
        for (int i = 0; i < POOL_SIZE; i++) {
            pool.add(Instance.builder(wasmModule).build());
        }
    }

    @PreDestroy
    void shutdown() {
        pool.clear();
    }

    /**
     * Generates puzzle SVG for a given mode.
     * @param configJson  JSON matching PuzzleConfig serde schema (without "mode" field)
     * @param mode        0–3 (see export_svg docs)
     * @return SVG string
     */
    public String generate(String configJson, int mode) {
        String json = injectMode(configJson, mode);
        byte[] input = json.getBytes(StandardCharsets.UTF_8);

        Instance inst;
        try {
            inst = pool.take();
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            throw new IllegalStateException("Interrupted waiting for Wasm instance", e);
        }

        try {
            ExportFunction getInputPtr = inst.export("get_input_ptr");
            ExportFunction getOutputPtr = inst.export("get_output_ptr");
            ExportFunction generatePuzzle = inst.export("generate_puzzle");

            int inputPtr = (int) getInputPtr.apply()[0];
            int outputPtr = (int) getOutputPtr.apply()[0];

            inst.memory().write(inputPtr, input);

            long outputLen = generatePuzzle.apply(input.length)[0];
            if (outputLen == 0) {
                throw new RuntimeException("Wasm generate_puzzle returned error (output_len=0)");
            }

            byte[] output = inst.memory().readBytes(outputPtr, (int) outputLen);
            return new String(output, StandardCharsets.UTF_8);
        } catch (com.dylibso.chicory.wasm.ChicoryException e) {
            throw new RuntimeException("Wasm execution failed", e);
        } finally {
            pool.offer(inst);
        }
    }

    public String getWasmSha256() {
        return wasmSha256;
    }

    private static String injectMode(String configJson, int mode) {
        String trimmed = configJson.strip();
        if (trimmed.endsWith("}")) {
            return trimmed.substring(0, trimmed.length() - 1) + ",\"mode\":" + mode + "}";
        }
        throw new IllegalArgumentException("Invalid config JSON: " + configJson);
    }

    private static String sha256hex(byte[] bytes) {
        try {
            var md = java.security.MessageDigest.getInstance("SHA-256");
            byte[] hash = md.digest(bytes);
            var sb = new StringBuilder(64);
            for (byte b : hash) sb.append(String.format("%02x", b));
            return sb.toString();
        } catch (java.security.NoSuchAlgorithmException e) {
            throw new IllegalStateException(e);
        }
    }
}
