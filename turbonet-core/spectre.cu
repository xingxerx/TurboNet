/**
 * SPECTRE-GPU: Polymorphic Payload Mutation Engine
 * 
 * GPU-accelerated payload obfuscation for penetration testing.
 * Generates thousands of unique payload variants in parallel,
 * selecting the highest-entropy variant for AV evasion.
 * 
 * Part of TurboNet Quantum-Hardened Security Toolkit
 */

#include <cuda_runtime.h>

// ============================================================================
// ENTROPY CALCULATION (Shannon Entropy)
// ============================================================================

/**
 * Calculate Shannon entropy for a byte sequence.
 * Higher entropy = more random-looking = better evasion.
 * 
 * @param data Pointer to byte sequence
 * @param len Length of sequence
 * @return Entropy value (0.0 - 8.0 for bytes)
 */
__device__ float calculate_entropy(const unsigned char* data, int len) {
    // Byte frequency histogram
    int counts[256];
    for (int i = 0; i < 256; i++) counts[i] = 0;
    
    // Count occurrences
    for (int i = 0; i < len; i++) {
        counts[data[i]]++;
    }
    
    // Calculate Shannon entropy: H = -Σ p(x) * log2(p(x))
    float entropy = 0.0f;
    float len_f = (float)len;
    
    for (int i = 0; i < 256; i++) {
        if (counts[i] > 0) {
            float p = (float)counts[i] / len_f;
            entropy -= p * log2f(p);
        }
    }
    
    return entropy;
}

// ============================================================================
// MUTATION KERNELS
// ============================================================================

/**
 * XOR mutation with thread-unique key derivation.
 * Each thread applies a different XOR pattern based on its ID.
 */
__device__ void mutate_xor(
    const unsigned char* input,
    unsigned char* output,
    int len,
    unsigned int thread_id,
    unsigned long long salt
) {
    // Derive unique key from thread ID and salt
    unsigned long long key = (thread_id * 0x5DEECE66DULL + salt) ^ 0xB;
    
    for (int i = 0; i < len; i++) {
        // Rolling XOR with key bytes
        unsigned char key_byte = (key >> ((i % 8) * 8)) & 0xFF;
        output[i] = input[i] ^ key_byte;
        
        // Key evolution for each byte
        key = key * 0x5851F42D4C957F2DULL + thread_id;
    }
}

/**
 * ROL/ROR (rotate left/right) mutation.
 * Rotates each byte by a varying amount.
 */
__device__ void mutate_rotate(
    const unsigned char* input,
    unsigned char* output,
    int len,
    unsigned int thread_id
) {
    int rotation = (thread_id % 7) + 1;  // 1-7 bit rotation
    bool rotate_left = (thread_id & 1);  // Even=left, Odd=right
    
    for (int i = 0; i < len; i++) {
        unsigned char b = input[i];
        if (rotate_left) {
            output[i] = (b << rotation) | (b >> (8 - rotation));
        } else {
            output[i] = (b >> rotation) | (b << (8 - rotation));
        }
    }
}

/**
 * Substitution mutation using lookup table.
 * Creates thread-unique S-box for byte substitution.
 */
__device__ void mutate_substitute(
    const unsigned char* input,
    unsigned char* output,
    int len,
    unsigned int thread_id,
    unsigned long long salt
) {
    // Generate thread-unique substitution table
    unsigned char sbox[256];
    unsigned long long seed = thread_id * salt + 0xDEADBEEF;
    
    // Initialize with identity mapping
    for (int i = 0; i < 256; i++) sbox[i] = (unsigned char)i;
    
    // Fisher-Yates shuffle using LCG
    for (int i = 255; i > 0; i--) {
        seed = seed * 0x5851F42D4C957F2DULL + 1;
        int j = (int)(seed % (i + 1));
        unsigned char tmp = sbox[i];
        sbox[i] = sbox[j];
        sbox[j] = tmp;
    }
    
    // Apply substitution
    for (int i = 0; i < len; i++) {
        output[i] = sbox[input[i]];
    }
}

/**
 * Combined mutation: XOR + ROL + SUB cascade.
 * Strongest obfuscation through layered transformations.
 */
__device__ void mutate_cascade(
    const unsigned char* input,
    unsigned char* output,
    int len,
    unsigned int thread_id,
    unsigned long long salt
) {
    unsigned char temp[4096];  // Stack buffer for intermediate results
    
    // Limit to stack buffer size
    int work_len = len < 4096 ? len : 4096;
    
    // Layer 1: XOR
    mutate_xor(input, temp, work_len, thread_id, salt);
    
    // Layer 2: Rotate
    mutate_rotate(temp, output, work_len, thread_id);
    
    // Layer 3: Final XOR pass with position-dependent key
    for (int i = 0; i < work_len; i++) {
        output[i] ^= ((thread_id + i) * 17) & 0xFF;
    }
}

// ============================================================================
// MAIN MUTATION KERNEL
// ============================================================================

/**
 * GPU Kernel: Generate polymorphic payload variants
 * 
 * Each thread creates one unique variant of the input payload
 * using a combination of XOR, rotation, and substitution mutations.
 * 
 * @param input      Original payload bytes
 * @param output     Buffer for all variants: [variant0][variant1]...
 * @param entropies  Entropy score per variant
 * @param len        Length of original payload
 * @param salt       Session entropy (from Kyber shared secret)
 * @param num_variants Total number of variants to generate
 * @param mutation_mode 0=XOR, 1=ROL, 2=SUB, 3=CASCADE
 */
extern "C" __global__ void spectre_mutate_kernel(
    const unsigned char* input,
    unsigned char* output,
    float* entropies,
    int len,
    unsigned long long salt,
    int num_variants,
    int mutation_mode
) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    
    if (idx >= num_variants) return;
    
    // Calculate output offset for this variant
    unsigned char* my_output = output + (idx * len);
    
    // Apply mutation based on mode
    switch (mutation_mode) {
        case 0:
            mutate_xor(input, my_output, len, idx, salt);
            break;
        case 1:
            mutate_rotate(input, my_output, len, idx);
            break;
        case 2:
            mutate_substitute(input, my_output, len, idx, salt);
            break;
        case 3:
        default:
            mutate_cascade(input, my_output, len, idx, salt);
            break;
    }
    
    // Calculate and store entropy for this variant
    entropies[idx] = calculate_entropy(my_output, len);
}

// ============================================================================
// REDUCTION KERNEL: Find highest-entropy variant
// ============================================================================

/**
 * GPU Kernel: Find variant with maximum entropy
 * 
 * Uses parallel reduction to efficiently find the best variant index.
 * 
 * @param entropies   Array of entropy values
 * @param num_variants Number of variants
 * @param best_index  Output: index of highest-entropy variant
 * @param best_entropy Output: highest entropy value
 */
extern "C" __global__ void spectre_find_best_kernel(
    const float* entropies,
    int num_variants,
    int* best_index,
    float* best_entropy
) {
    __shared__ float shared_entropy[256];
    __shared__ int shared_index[256];
    
    int tid = threadIdx.x;
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    
    // Load into shared memory
    if (idx < num_variants) {
        shared_entropy[tid] = entropies[idx];
        shared_index[tid] = idx;
    } else {
        shared_entropy[tid] = -1.0f;
        shared_index[tid] = -1;
    }
    __syncthreads();
    
    // Parallel reduction
    for (int stride = blockDim.x / 2; stride > 0; stride >>= 1) {
        if (tid < stride) {
            if (shared_entropy[tid + stride] > shared_entropy[tid]) {
                shared_entropy[tid] = shared_entropy[tid + stride];
                shared_index[tid] = shared_index[tid + stride];
            }
        }
        __syncthreads();
    }
    
    // Thread 0 writes result
    if (tid == 0) {
        // Atomic max for multi-block reduction
        // Note: This is a simplified version; full implementation would use
        // atomic operations for true multi-block reduction
        if (shared_entropy[0] > *best_entropy) {
            *best_entropy = shared_entropy[0];
            *best_index = shared_index[0];
        }
    }
}

// ============================================================================
// DECODER KERNEL (for receiver-side)
// ============================================================================

/**
 * GPU Kernel: Decode mutated payload back to original
 * 
 * Must use the same thread_id and salt that was used during mutation.
 * This allows secure transmission where only the receiver knows the
 * mutation parameters.
 * 
 * @param input       Mutated payload bytes
 * @param output      Decoded original bytes
 * @param len         Length of payload
 * @param salt        Same salt used during mutation
 * @param thread_id   Same thread_id used during mutation
 * @param mutation_mode Same mode used during mutation
 */
extern "C" __global__ void spectre_decode_kernel(
    const unsigned char* input,
    unsigned char* output,
    int len,
    unsigned long long salt,
    int thread_id,
    int mutation_mode
) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx >= len) return;
    
    // For XOR-based mutations, applying the same operation decodes
    // For other modes, inverse operations would be needed
    if (mutation_mode == 0 || mutation_mode == 3) {
        // XOR is its own inverse
        unsigned long long key = (thread_id * 0x5DEECE66DULL + salt) ^ 0xB;
        for (int i = 0; i < idx; i++) {
            key = key * 0x5851F42D4C957F2DULL + thread_id;
        }
        unsigned char key_byte = (key >> ((idx % 8) * 8)) & 0xFF;
        
        if (mutation_mode == 3) {
            // Reverse cascade: undo final XOR, then main XOR
            unsigned char temp = input[idx] ^ (((thread_id + idx) * 17) & 0xFF);
            // Undo rotation (reverse of what mutate_rotate does)
            int rotation = (thread_id % 7) + 1;
            bool rotate_left = (thread_id & 1);
            if (rotate_left) {
                temp = (temp >> rotation) | (temp << (8 - rotation));
            } else {
                temp = (temp << rotation) | (temp >> (8 - rotation));
            }
            output[idx] = temp ^ key_byte;
        } else {
            output[idx] = input[idx] ^ key_byte;
        }
    }
    }
}

// ============================================================================
// TERRAIN GENERATION KERNEL
// ============================================================================

/**
 * Simple pseudo-random hash for noise generation
 */
__device__ float hash2d(float x, float y) {
    float prod = x * 12.9898 + y * 78.233;
    float sn = sinf(prod);
    return sn * 43758.5453 - floorf(sn * 43758.5453);
}

/**
 * Bilinear interpolation
 */
__device__ float lerp_f(float a, float b, float t) {
    return a + t * (b - a);
}

/**
 * Value Noise 2D
 */
__device__ float value_noise(float x, float y) {
    float i_x = floorf(x);
    float i_y = floorf(y);
    float f_x = x - i_x;
    float f_y = y - i_y;
    
    // Four corners
    float a = hash2d(i_x,     i_y);
    float b = hash2d(i_x + 1.0, i_y);
    float c = hash2d(i_x,     i_y + 1.0);
    float d = hash2d(i_x + 1.0, i_y + 1.0);
    
    // Smooth interpolation
    float u_x = f_x * f_x * (3.0 - 2.0 * f_x);
    float u_y = f_y * f_y * (3.0 - 2.0 * f_y);
    
    return lerp_f(lerp_f(a, b, u_x), lerp_f(c, d, u_x), u_y);
}

/**
 * Fractal Brownian Motion (FBM) for terrain detail
 */
__device__ float fbm(float x, float y, int octaves) {
    float total = 0.0f;
    float amplitude = 1.0f;
    float frequency = 1.0f;
    float max_value = 0.0f;  // Used for normalizing result to 0.0 - 1.0

    for(int i = 0; i < octaves; i++) {
        total += value_noise(x * frequency, y * frequency) * amplitude;
        max_value += amplitude;
        
        amplitude *= 0.5f;
        frequency *= 2.0f;
    }
    
    return total / max_value;
}

/**
 * GPU Kernel: Generate Terrain Heightmap
 * 
 * Generates a heightmap for a world based on a seed.
 * 
 * @param heightmaps  Output array for all generated worlds [world_idx][y][x]
 * @param width       Width of the heightmap
 * @param height      Height of the heightmap
 * @param num_worlds  Number of worlds to generate (batch size)
 * @param seeds       Array of seeds for each world
 */
extern "C" __global__ void terrain_gen_kernel(
    float* heightmaps,
    int width,
    int height,
    int num_worlds,
    unsigned long long* seeds
) {
    // Current World Index
    int world_idx = blockIdx.z; 
    if (world_idx >= num_worlds) return;
    
    // Current Pixel Coordinates
    int x = blockIdx.x * blockDim.x + threadIdx.x;
    int y = blockIdx.y * blockDim.y + threadIdx.y;
    
    if (x >= width || y >= height) return;
    
    // Calculate global index in the massive array
    // Flat index = world_offset + row_offset + col_offset
    int flat_idx = (world_idx * width * height) + (y * width) + x;
    
    // Use world-specific seed
    unsigned long long seed = seeds[world_idx];
    
    // Coordinate scaling
    float scale = 0.02f;
    float px = (x + (seed % 1000)) * scale;
    float py = (y + (seed / 1000)) * scale;
    
    // Generate height using FBM (4 octaves)
    float h = fbm(px, py, 4);
    
    // Apply some non-linear shaping for ridges
    heightmaps[flat_idx] = powf(h, 1.2f) * 50.0f; // Scale to 0-50 meters
}
