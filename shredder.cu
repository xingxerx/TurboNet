extern "C" __global__ void shred_kernel(
    const unsigned char* input, 
    unsigned char* band24, 
    unsigned char* band5g1, 
    unsigned char* band5g2, 
    unsigned long long n,
    unsigned long long salt, // Derived from Shared Secret
    unsigned long long w0, unsigned long long w1, unsigned long long w2,
    unsigned long long i0, unsigned long long i1, unsigned long long i2) 
{
    unsigned long long idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < n) {
        unsigned long long W = w0 + w1 + w2;
        unsigned long long effective_idx = idx + (salt % W); 
        unsigned long long block_id = effective_idx / W;
        unsigned long long pos_in_block = effective_idx % W;
        
        // LEVEL 12: XOR obfuscation using salt for in-kernel scrambling
        unsigned char obfuscated = input[idx] ^ ((salt >> (idx % 8)) & 0xFF);
        
        // Asymmetric distribution based on AI weights
        if (pos_in_block < w0) {
            band24[block_id * w0 + pos_in_block - i0] = obfuscated;
        } else if (pos_in_block < w0 + w1) {
            band5g1[block_id * w1 + (pos_in_block - w0) - i1] = obfuscated;
        } else {
            band5g2[block_id * w2 + (pos_in_block - w0 - w1) - i2] = obfuscated;
        }
    }
}

// LEVEL 12: De-obfuscation kernel for receiver side
extern "C" __global__ void unshred_kernel(
    const unsigned char* band24,
    const unsigned char* band5g1,
    const unsigned char* band5g2,
    unsigned char* output,
    unsigned long long n,
    unsigned long long salt,
    unsigned long long w0, unsigned long long w1, unsigned long long w2)
{
    unsigned long long idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < n) {
        unsigned long long W = w0 + w1 + w2;
        unsigned long long effective_idx = idx + (salt % W);
        unsigned long long block_id = effective_idx / W;
        unsigned long long pos_in_block = effective_idx % W;
        
        unsigned char obfuscated;
        if (pos_in_block < w0) {
            obfuscated = band24[block_id * w0 + pos_in_block];
        } else if (pos_in_block < w0 + w1) {
            obfuscated = band5g1[block_id * w1 + (pos_in_block - w0)];
        } else {
            obfuscated = band5g2[block_id * w2 + (pos_in_block - w0 - w1)];
        }
        
        // De-obfuscate using same salt
        output[idx] = obfuscated ^ ((salt >> (idx % 8)) & 0xFF);
    }
}