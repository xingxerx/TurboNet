// TurboNet Forward Error Correction (FEC) Module
// Provides erasure coding for packet loss recovery

use std::error::Error;

/// Results of encoding a data block
pub struct EncodedShards {
    /// Original data shards
    pub data_shards: Vec<Vec<u8>>,
    /// Parity shards for recovery
    pub parity_shards: Vec<Vec<u8>>,
    /// Size of each shard
    pub shard_size: usize,
}

/// FEC Encoder using Reed-Solomon erasure coding
pub struct ReedSolomonFec {
    encoder: reed_solomon_erasure::ReedSolomon<reed_solomon_erasure::galois_8::Field>,
    data_shards: usize,
    parity_shards: usize,
}

impl ReedSolomonFec {
    /// Create a new FEC encoder
    /// - data_shards: Number of original data chunks
    /// - parity_shards: Number of parity chunks for recovery
    /// 
    /// Can recover from loss of up to `parity_shards` chunks
    pub fn new(data_shards: usize, parity_shards: usize) -> Result<Self, Box<dyn Error>> {
        let encoder = reed_solomon_erasure::ReedSolomon::new(data_shards, parity_shards)?;
        Ok(Self {
            encoder,
            data_shards,
            parity_shards,
        })
    }
    
    /// Encode data into shards with parity
    /// Returns data shards + parity shards
    pub fn encode(&self, data: &[u8]) -> Result<EncodedShards, Box<dyn Error>> {
        let total_shards = self.data_shards + self.parity_shards;
        let shard_size = (data.len() + self.data_shards - 1) / self.data_shards;
        
        // Pad data to fill all shards evenly
        let padded_len = shard_size * self.data_shards;
        let mut padded_data = data.to_vec();
        padded_data.resize(padded_len, 0);
        
        // Split into shards
        let mut shards: Vec<Vec<u8>> = padded_data
            .chunks(shard_size)
            .map(|c| c.to_vec())
            .collect();
        
        // Add empty parity shards
        for _ in 0..self.parity_shards {
            shards.push(vec![0u8; shard_size]);
        }
        
        // Calculate parity
        self.encoder.encode(&mut shards)?;
        
        let data_shards: Vec<Vec<u8>> = shards.iter().take(self.data_shards).cloned().collect();
        let parity_shards: Vec<Vec<u8>> = shards.iter().skip(self.data_shards).cloned().collect();
        
        Ok(EncodedShards {
            data_shards,
            parity_shards,
            shard_size,
        })
    }
    
    /// Decode data from shards (some may be missing)
    /// Pass None for lost shards
    pub fn decode(
        &self,
        shards: &[Option<Vec<u8>>],
        original_len: usize,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut shards_mut: Vec<Option<Vec<u8>>> = shards.to_vec();
        
        // Reconstruct missing shards
        self.encoder.reconstruct(&mut shards_mut)?;
        
        // Extract data shards
        let mut result = Vec::with_capacity(original_len);
        for shard in shards_mut.iter().take(self.data_shards) {
            if let Some(data) = shard {
                result.extend_from_slice(data);
            }
        }
        
        // Truncate to original length
        result.truncate(original_len);
        Ok(result)
    }
    
    /// Get the number of data shards
    pub fn data_shards(&self) -> usize {
        self.data_shards
    }
    
    /// Get the number of parity shards
    pub fn parity_shards(&self) -> usize {
        self.parity_shards
    }
    
    /// Get total number of shards
    pub fn total_shards(&self) -> usize {
        self.data_shards + self.parity_shards
    }
}

/// High-level FEC configuration for different loss scenarios
pub mod presets {
    use super::*;
    
    /// Standard: 10 data + 3 parity (30% overhead, recovers 3 lost)
    pub fn standard() -> Result<ReedSolomonFec, Box<dyn Error>> {
        ReedSolomonFec::new(10, 3)
    }
    
    /// High resilience: 10 data + 5 parity (50% overhead, recovers 5 lost)
    pub fn high_resilience() -> Result<ReedSolomonFec, Box<dyn Error>> {
        ReedSolomonFec::new(10, 5)
    }
    
    /// Low overhead: 20 data + 2 parity (10% overhead, recovers 2 lost)
    pub fn low_overhead() -> Result<ReedSolomonFec, Box<dyn Error>> {
        ReedSolomonFec::new(20, 2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_no_loss() {
        let fec = ReedSolomonFec::new(4, 2).unwrap();
        let data = b"Hello, TurboNet! This is a test of FEC encoding.";
        
        let encoded = fec.encode(data).unwrap();
        assert_eq!(encoded.data_shards.len(), 4);
        assert_eq!(encoded.parity_shards.len(), 2);
        
        // All shards present
        let mut all_shards: Vec<Option<Vec<u8>>> = encoded.data_shards.into_iter()
            .chain(encoded.parity_shards)
            .map(Some)
            .collect();
        
        let decoded = fec.decode(&all_shards, data.len()).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_encode_decode_with_loss() {
        let fec = ReedSolomonFec::new(4, 2).unwrap();
        let data = b"Hello, TurboNet! This is a test of FEC encoding.";
        
        let encoded = fec.encode(data).unwrap();
        
        // Simulate loss of 2 shards
        let mut shards: Vec<Option<Vec<u8>>> = encoded.data_shards.into_iter()
            .chain(encoded.parity_shards)
            .map(Some)
            .collect();
        shards[0] = None; // Lost first data shard
        shards[3] = None; // Lost fourth data shard
        
        let decoded = fec.decode(&shards, data.len()).unwrap();
        assert_eq!(decoded, data);
    }
}
