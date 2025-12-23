extern "C" __global__ void shred_kernel(
    const unsigned char* input, 
    unsigned char* band24, 
    unsigned char* band5g1, 
    unsigned char* band5g2, 
    unsigned long long n,
    unsigned long long salt,
    unsigned long long w0, 
    unsigned long long w1, 
    unsigned long long w2,
    unsigned long long i0,
    unsigned long long i1,
    unsigned long long i2) 
{
    unsigned long long idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx < n) {
        unsigned long long W = w0 + w1 + w2;
        unsigned long long pattern_offset = (salt % W);
        unsigned long long effective_idx = idx + pattern_offset;
        unsigned long long block_id = effective_idx / W;
        unsigned long long pos_in_block = effective_idx % W;
        
        // Prevent writing past the allocated buffers
        if (pos_in_block < w0) {
            band24[block_id * w0 + pos_in_block] = input[idx];
        } else if (pos_in_block < w0 + w1) {
            band5g1[block_id * w1 + (pos_in_block - w0)] = input[idx];
        } else {
            band5g2[block_id * w2 + (pos_in_block - w0 - w1)] = input[idx];
        }
    }
}